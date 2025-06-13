use anchor_client::{
    Cluster,
    solana_client::{
        rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient},
        rpc_config::RpcTransactionConfig,
    },
    solana_sdk::{bs58, commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature},
};
use anchor_lang::{declare_program, AnchorDeserialize};
use solana_transaction_status_client_types::UiTransactionEncoding;
use std::str::FromStr;
use anchor_lang::__private::base64::Engine;
use anchor_lang::__private::base64::prelude::BASE64_STANDARD;
use anyhow::anyhow;

const PROGRAM_DATA: &'static str = "Program data: ";
declare_program!(red_packet);
use red_packet::events::{
    ExpiryTimeUpdated, RedPacketClaimed, RedPacketCreated, RedPacketRefunded,
};
use crate::red_packet::client::args::CreateRedpacket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let rpc_url = "https://api.mainnet-beta.solana.com"; // 主网
    let rpc_url = "https://api.devnet.solana.com"; // 开发网
    let rpc_client =
        RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let program_id: Pubkey = "HqSDjxnoR35q8uRMG3LDDvbJ9Hqj4H4bWMPAsBF1hqJq".parse()?;
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

            println!("{:?}", logs);
            for log in logs {
                if log.starts_with(PROGRAM_DATA) {
                    let data_str = log.strip_prefix(PROGRAM_DATA).unwrap();
                    // let decoded_data = bs58::decode(data_str).into_vec()?;

                    // 解码 base64
                    let decoded_data = BASE64_STANDARD.decode(data_str)?;

                    println!("Decoded data (hex): {:?}", &decoded_data);

                    let (discriminator, event_data) = decoded_data.split_at(8);
                    println!("Event discriminator: {:?}", discriminator);
                    
                    
                    // discriminator 比较
                    
                    // 解析事件数据
                    let event = RedPacketCreated::try_from_slice(event_data)
                        .expect("Failed to deserialize event");

                    // 输出结果
                    println!("Decoded Event: {:?}", event);

                }
            }
        }
    }
    Ok(())
}
