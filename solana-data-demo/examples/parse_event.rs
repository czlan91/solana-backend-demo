use anchor_client::{
    solana_client::{
        rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient},
        rpc_config::RpcTransactionConfig,
    },
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature},
    Cluster,
};
use anchor_lang::__private::base64::prelude::BASE64_STANDARD;
use anchor_lang::__private::base64::Engine;
use anchor_lang::declare_program;
use solana_transaction_status_client_types::UiTransactionEncoding;
use std::str::FromStr;

const PROGRAM_DATA: &'static str = "Program data: ";
declare_program!(red_packet);
use crate::red_packet::utils::Event;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let rpc_url = "https://api.mainnet-beta.solana.com"; // 主网
    let rpc_url = Cluster::Devnet.url(); // 开发网
    let rpc_client =
        RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let program_id: Pubkey = "7rSdaJc2nJafXjKD39nxmhkmCexUFQsCisg42oyRsqvt".parse()?;
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

        println!("===========>    {:?}",tx.transaction.transaction);
        if let Some(meta) = tx.transaction.meta {
            println!("\n--- 交易 {} 的日志 ---", sig.signature);

            let logs = meta
                .log_messages
                .ok_or("No log messages")
                .map_err(|_| anyhow::anyhow!("No log messages"))?;

            println!("{:?}", logs);
            for log in logs {
                if log.starts_with(PROGRAM_DATA) {
                    let data_str = log.strip_prefix(PROGRAM_DATA).unwrap();
                    // let decoded_data = bs58::decode(data_str).into_vec()?;

                    // 解码 base64
                    let decoded_data = BASE64_STANDARD.decode(data_str)?;

                    println!("Decoded data (hex): {:?}", &decoded_data);
                    // discriminator 比较
                    match Event::try_from_bytes(&decoded_data) {
                        Ok(Event::RedPacketCreated(my_event)) => {
                            println!("RedPacketCreated Event: {:?}", my_event)
                        }
                        Ok(Event::RedPacketClaimed(my_event)) => {
                            println!("RedPacketClaimed Event: {:?}", my_event)
                        }
                        _ => {
                            println!("Not a valid event")
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
