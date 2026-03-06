// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use alloy::{
    network::EthereumWallet,
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};
use eyre::{Result, WrapErr};
use std::process::{Child, Command, Stdio};

/// Default dev account private key for anvil-polkadot.
pub const PVM_DEVNET_PRIVATE_KEY: &str =
    "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/// Manage an anvil-polkadot node for deploying PVM contracts.
/// The node is started as a child process and killed when this struct is dropped.
pub struct PvmNode {
    process: Option<Child>,
    rpc: String,
}

impl PvmNode {
    /// Starts a new anvil-polkadot process on a random available port.
    /// This node will be shut down when this struct is dropped.
    pub async fn new() -> Result<Self> {
        let port = find_free_port()?;
        let process = Command::new("anvil-polkadot")
            .args(["--port", &port.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err("failed to start anvil-polkadot (is it installed?)")?;

        let rpc = format!("http://127.0.0.1:{port}");
        wait_for_rpc(&rpc).await?;

        Ok(Self {
            process: Some(process),
            rpc,
        })
    }

    /// Get the anvil-polkadot node RPC URL.
    pub fn rpc(&self) -> &str {
        &self.rpc
    }

    /// Create a provider with the dev account keys to send requests to the node.
    pub async fn create_provider(&self) -> Result<impl Provider> {
        let signer: PrivateKeySigner = PVM_DEVNET_PRIVATE_KEY
            .parse()
            .expect("failed to parse pvm devnet private key");
        let wallet = EthereumWallet::from(signer);
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect(self.rpc())
            .await?;
        Ok(provider)
    }
}

impl Drop for PvmNode {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

/// Find an available TCP port by binding to port 0.
fn find_free_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .wrap_err("failed to bind to find free port")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

/// Poll the RPC endpoint until it responds to eth_blockNumber.
async fn wait_for_rpc(rpc: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let body = r#"{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}"#;

    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let resp = client
            .post(rpc)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await;
        if let Ok(resp) = resp {
            if let Ok(text) = resp.text().await {
                if text.contains("result") {
                    return Ok(());
                }
            }
        }
    }

    Err(eyre::eyre!(
        "anvil-polkadot did not become ready at {rpc} within 10s"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pvm_node_starts_and_responds() -> Result<()> {
        let node = PvmNode::new().await?;
        let provider = node.create_provider().await?;
        let block_number = provider.get_block_number().await?;
        assert!(block_number >= 0);
        Ok(())
    }
}
