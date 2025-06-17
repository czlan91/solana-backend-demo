use anyhow::Result;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status_client_types::UiTransactionEncoding;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let client = RpcClient::new_with_commitment(
        anchor_client::Cluster::Devnet.url().parse()?,
        CommitmentConfig::confirmed(),
    );

    let tx_sig = Signature::from_str(
        "4273notKqSB1aAKqCws4TqZHBFGb3DgsFTnYVbZetXbpPNh2knA4jQa7EzBJysnXbpiMTcgT4HMfkZFBANF3sazD",
    )?;

    let config = RpcTransactionConfig {
        commitment: CommitmentConfig::finalized().into(),
        encoding: UiTransactionEncoding::Base64.into(),
        max_supported_transaction_version: Some(0),
    };

    let transaction = client.get_transaction_with_config(&tx_sig, config).await?;

    println!("{:#?}", transaction);

    Ok(())
}