use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::signature::Signature;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use solana_transaction_status_client_types::UiTransactionEncoding;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let rpc_url = "https://api.mainnet-beta.solana.com"; // 主网
    let rpc_url = "https://api.devnet.solana.com"; // 开发网
    let rpc_client =
        RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let program_id: Pubkey = "HUkozHHVBLsn1w6SDYZa5fkWv3iHG5wjfLXM2b6j7pKC".parse()?;
    // todo 可以通过循环，向前找交易。每次 记录 第一个交易，直到找到一个没有日志的
    let config = GetConfirmedSignaturesForAddress2Config {
        before: None,
        until: None,
        limit: Some(100),
        commitment: Some(CommitmentConfig::confirmed()),
    };
    let mut signatures = rpc_client.get_signatures_for_address_with_config(&program_id, config)?;

    // 反转 signatures
    signatures.reverse();
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };

    for sig in signatures {
        let tx = rpc_client
            .get_transaction_with_config(&Signature::from_str(&sig.signature)?, config)?;

        if let Some(meta) = tx.transaction.meta {
            println!("\n--- 交易 {} 的日志 ---", sig.signature);

            let logs = meta
                .log_messages
                .ok_or("No log messages")
                .map_err(|_| anyhow::anyhow!("No log messages"))?;
            if logs.join(",").contains(&"Instruction:".to_string()) {
                // todo 写到日志里面，
                println!("{:?}", logs);
            }
        }
    }
    Ok(())
}
