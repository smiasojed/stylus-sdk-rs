// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#[cfg(feature = "integration-tests")]
mod pvm_factory_test {
    use alloy::{
        primitives::{Address, FixedBytes, U256},
        sol,
    };
    use eyre::Result;
    use stylus_tools::devnet::PvmNode;
    use stylus_tools::PvmDeployer;

    // Note: We use raw call encoding instead of sol! for the factory because
    // the function name "deploy" conflicts with alloy's generated deploy() method.
    sol! {
        #[sol(rpc)]
        interface ICounter {
            function get() external view returns (uint256);
            function setCount(uint256 count) external;
            function increment() external;
        }
    }

    /// Encode a call to `deploy(bytes32)` — selector 0x4fd7e246 (from `cast sig "deploy(bytes32)"`)
    fn encode_deploy_call(code_hash: FixedBytes<32>) -> Vec<u8> {
        use alloy::sol_types::SolCall;
        alloy::sol! {
            function deploy(bytes32 code_hash) external returns (address);
        }
        deployCall { code_hash }.abi_encode()
    }

    /// Encode a call to `deploy2(bytes32,bytes32)`
    fn encode_deploy2_call(code_hash: FixedBytes<32>, salt: FixedBytes<32>) -> Vec<u8> {
        use alloy::sol_types::SolCall;
        alloy::sol! {
            function deploy2(bytes32 code_hash, bytes32 salt) external returns (address);
        }
        deploy2Call { code_hash, salt }.abi_encode()
    }

    /// Decode an address from ABI-encoded return data
    fn decode_address(data: &[u8]) -> Address {
        assert!(data.len() >= 32, "return data too short");
        Address::from_slice(&data[12..32])
    }

    #[tokio::test]
    async fn test_create1() -> Result<()> {
        use alloy::{network::TransactionBuilder, providers::Provider, rpc::types::TransactionRequest};

        let node = PvmNode::new().await?;
        let provider = node.create_provider().await?;

        // Deploy counter bytecode to the chain (uploads code)
        let counter_deployer = PvmDeployer::builder()
            .rpc(node.rpc())
            .dir("../revive_counter".to_owned())
            .build();
        let counter_bytecode = counter_deployer.build_pvm()?;
        let code_hash = PvmDeployer::code_hash(&counter_bytecode);
        let (_, _) = counter_deployer.deploy().await?;

        // Deploy factory contract
        let (factory_addr, _) = PvmDeployer::builder()
            .rpc(node.rpc())
            .dir("../revive_factory".to_owned())
            .build()
            .deploy()
            .await?;

        // CREATE1: predict address via eth_call
        let calldata = encode_deploy_call(FixedBytes::from(code_hash));
        let tx = TransactionRequest::default()
            .with_to(factory_addr)
            .with_input(calldata.clone());
        let result = provider.call(tx).await?;
        let counter_addr = decode_address(&result);

        // Actually deploy via send
        let tx = TransactionRequest::default()
            .with_to(factory_addr)
            .with_input(calldata);
        provider
            .send_transaction(tx)
            .await?
            .watch()
            .await?;

        // Verify the deployed counter works
        let counter = ICounter::ICounterInstance::new(counter_addr, &provider);

        let val: U256 = counter.get().call().await?;
        assert_eq!(val, U256::ZERO, "initial counter should be 0");

        counter
            .setCount(U256::from(99))
            .send()
            .await?
            .watch()
            .await?;
        let val: U256 = counter.get().call().await?;
        assert_eq!(val, U256::from(99), "counter should be 99 after setCount");

        counter.increment().send().await?.watch().await?;
        let val: U256 = counter.get().call().await?;
        assert_eq!(val, U256::from(100), "counter should be 100 after increment");

        Ok(())
    }

    #[tokio::test]
    async fn test_create2() -> Result<()> {
        use alloy::{network::TransactionBuilder, providers::Provider, rpc::types::TransactionRequest};

        let node = PvmNode::new().await?;
        let provider = node.create_provider().await?;

        // Deploy counter bytecode to the chain
        let counter_deployer = PvmDeployer::builder()
            .rpc(node.rpc())
            .dir("../revive_counter".to_owned())
            .build();
        let counter_bytecode = counter_deployer.build_pvm()?;
        let code_hash = PvmDeployer::code_hash(&counter_bytecode);
        let (_, _) = counter_deployer.deploy().await?;

        // Deploy factory
        let (factory_addr, _) = PvmDeployer::builder()
            .rpc(node.rpc())
            .dir("../revive_factory".to_owned())
            .build()
            .deploy()
            .await?;

        let salt1 = FixedBytes::<32>::with_last_byte(1);
        let salt2 = FixedBytes::<32>::with_last_byte(2);

        // CREATE2 with salt1: predict address
        let calldata = encode_deploy2_call(FixedBytes::from(code_hash), salt1);
        let tx = TransactionRequest::default()
            .with_to(factory_addr)
            .with_input(calldata.clone());
        let result = provider.call(tx).await?;
        let addr1 = decode_address(&result);

        // Actually deploy with salt1
        let tx = TransactionRequest::default()
            .with_to(factory_addr)
            .with_input(calldata);
        provider.send_transaction(tx).await?.watch().await?;

        // Verify deployed counter works
        let counter1 = ICounter::ICounterInstance::new(addr1, &provider);
        let val: U256 = counter1.get().call().await?;
        assert_eq!(val, U256::ZERO, "CREATE2 counter should start at 0");

        counter1
            .setCount(U256::from(42))
            .send()
            .await?
            .watch()
            .await?;
        let val: U256 = counter1.get().call().await?;
        assert_eq!(val, U256::from(42), "CREATE2 counter should be 42");

        // CREATE2 with salt2: different salt → different address
        let calldata = encode_deploy2_call(FixedBytes::from(code_hash), salt2);
        let tx = TransactionRequest::default()
            .with_to(factory_addr)
            .with_input(calldata.clone());
        let result = provider.call(tx).await?;
        let addr2 = decode_address(&result);
        assert_ne!(addr1, addr2, "different salts should produce different addresses");

        // Deploy with salt2
        let tx = TransactionRequest::default()
            .with_to(factory_addr)
            .with_input(calldata);
        provider.send_transaction(tx).await?.watch().await?;

        // Verify second counter is isolated (starts at 0, not 42)
        let counter2 = ICounter::ICounterInstance::new(addr2, &provider);
        let val: U256 = counter2.get().call().await?;
        assert_eq!(val, U256::ZERO, "second CREATE2 counter should be independent");

        // Verify first counter still has its state
        let val: U256 = counter1.get().call().await?;
        assert_eq!(val, U256::from(42), "first counter state should persist");

        Ok(())
    }
}
