// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{Address, Bytes, TxHash, B256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use eyre::{bail, Result, WrapErr};
use std::{env, path::Path, process::Command};
use tiny_keccak::{Hasher, Keccak};
use typed_builder::TypedBuilder;

/// Deploys PVM (PolkaVM) contracts to anvil-polkadot by building the contract
/// and sending the raw bytecode as a CREATE transaction.
///
/// This mirrors the WASM [`Deployer`](crate::Deployer) but for PVM targets.
#[derive(TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct PvmDeployer {
    rpc: String,

    #[cfg_attr(
        feature = "integration-tests",
        builder(default = crate::devnet::pvm_node::PVM_DEVNET_PRIVATE_KEY.to_owned())
    )]
    #[cfg_attr(not(feature = "integration-tests"), builder())]
    private_key: String,

    #[builder(default)]
    dir: Option<String>,
}

impl PvmDeployer {
    /// Build PVM contract and deploy raw bytecode.
    /// Returns `(deployed_address, tx_hash)`.
    pub async fn deploy(&self) -> Result<(Address, TxHash)> {
        let bytecode = self.build_pvm()?;
        let provider = self.create_provider().await?;

        let tx = TransactionRequest::default().with_input(Bytes::from(bytecode));
        let receipt = provider
            .send_transaction(tx)
            .await
            .wrap_err("failed to send PVM deployment tx")?
            .get_receipt()
            .await
            .wrap_err("failed to get PVM deployment receipt")?;

        let address = receipt
            .contract_address
            .ok_or_else(|| eyre::eyre!("no contract address in deployment receipt"))?;

        Ok((address, receipt.transaction_hash))
    }

    /// Build the PVM contract and return the raw `.polkavm` bytecode.
    pub fn build_pvm(&self) -> Result<Vec<u8>> {
        let original_dir = env::current_dir()?;
        if let Some(dir) = &self.dir {
            env::set_current_dir(Path::new(dir))
                .wrap_err_with(|| format!("failed to cd to {dir}"))?;
        }

        let output = Command::new("cargo")
            .args(["stylus", "build", "--target", "pvm"])
            .output()
            .wrap_err("failed to run cargo stylus build --target pvm")?;

        env::set_current_dir(&original_dir)?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!("cargo stylus build --target pvm failed: {err}");
        }

        // Extract the .polkavm path from the build output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let all_output = format!("{stdout}\n{stderr}");

        let polkavm_path = extract_polkavm_path(&all_output)
            .ok_or_else(|| eyre::eyre!("could not find .polkavm path in build output"))?;

        std::fs::read(&polkavm_path)
            .wrap_err_with(|| format!("failed to read polkavm file at {polkavm_path}"))
    }

    /// Compute code_hash = keccak256(bytecode), used for factory pattern.
    pub fn code_hash(bytecode: &[u8]) -> B256 {
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(bytecode);
        hasher.finalize(&mut output);
        B256::from(output)
    }

    async fn create_provider(&self) -> Result<impl Provider> {
        let signer: PrivateKeySigner = self
            .private_key
            .parse()
            .wrap_err("failed to parse private key")?;
        let wallet = EthereumWallet::from(signer);
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect(&self.rpc)
            .await?;
        Ok(provider)
    }
}

/// Extract the .polkavm file path from cargo stylus build output.
/// Looks for a line containing a path ending in `.polkavm`.
fn extract_polkavm_path(output: &str) -> Option<String> {
    for line in output.lines() {
        // The build output contains lines like:
        // "Created /path/to/target/.../contract_name.polkavm (12345 bytes)"
        // or "Pvm contract built: /path/to/contract.polkavm"
        if let Some(start) = line.find('/') {
            let rest = &line[start..];
            if let Some(end) = rest.find(".polkavm") {
                return Some(rest[..end + ".polkavm".len()].to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_polkavm_path() {
        let output = r#"Building PVM contract: revive_counter
Created /tmp/target/riscv64emac-unknown-none-polkavm/release/revive_counter.polkavm (29476 bytes)
Pvm contract built: /tmp/target/riscv64emac-unknown-none-polkavm/release/revive_counter.polkavm"#;
        let path = extract_polkavm_path(output);
        assert_eq!(
            path,
            Some("/tmp/target/riscv64emac-unknown-none-polkavm/release/revive_counter.polkavm".to_string())
        );
    }

    #[test]
    fn test_code_hash() {
        let bytecode = b"hello";
        let hash = PvmDeployer::code_hash(bytecode);
        assert_ne!(hash, B256::ZERO);
    }
}
