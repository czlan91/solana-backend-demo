use anchor_client::Cluster;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_lang::__private::base64::Engine;
use anchor_lang::__private::base64::prelude::BASE64_STANDARD;
use anchor_lang::declare_program;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;

const PROGRAM_DATA: &str = "Program data: ";

const PUBKEY_REDPACKET: &str = "HqSDjxnoR35q8uRMG3LDDvbJ9Hqj4H4bWMPAsBF1hqJq";
const RECONNECT_DELAY: Duration = Duration::from_secs(3);
declare_program!(red_packet);
use crate::red_packet::utils::Event;
#[tokio::main]
async fn main() -> Result<()> {
    let url = Cluster::Devnet;

    let mut last_sig: Option<String> = None;
    let mut processed: HashSet<String> = HashSet::new();
    loop {
        match run_subscription(&url, &mut last_sig, &mut processed).await {
            Ok(_) => break,
            Err(e) => {
                eprintln!(
                    "连接中断: {:?}, {} 秒后重连...",
                    e,
                    RECONNECT_DELAY.as_secs()
                );
                sleep(RECONNECT_DELAY).await;
            }
        }
    }
    Ok(())
}

async fn run_subscription(
    url: &Cluster,
    last_sig: &mut Option<String>,
    processed: &mut HashSet<String>,
) -> Result<()> {
    let filter = RpcTransactionLogsFilter::Mentions(vec![PUBKEY_REDPACKET.to_string()]);
    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::confirmed()),
    };
    let client = PubsubClient::new(url.ws_url()).await?;

    let (mut stream, _unsubscribe) = client
        .logs_subscribe(filter, config)
        .await
        .context("创建 WebSocket 订阅失败")?;

    println!("✅ 已连接至 {}，监听日志中...", url);
    // 2. 补拉历史日志
    fill_missing_logs(url, last_sig, processed).await?;

    println!("开始实时监听日志...");

    while let Some(message) = stream.next().await {
        let logs = message.value.logs;
        for log in logs {
            if let Some(data_str) = log.strip_prefix(PROGRAM_DATA) {
                // 按你的逻辑 base64 解码和事件处理
                // 解码 base64
                let decoded_data = BASE64_STANDARD.decode(data_str)?;

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
    Err(anyhow::anyhow!("WebSocket连接断开"))
}

// 补拉历史日志（断线期间的 signature）
async fn fill_missing_logs(
    url: &Cluster,
    last_sig: &mut Option<String>,
    _processed: &mut HashSet<String>,
) -> Result<()> {
    let _rpc = RpcClient::new_with_commitment(url.url().to_string(), CommitmentConfig::confirmed());
    let _before = last_sig.clone().ok_or(anyhow::anyhow!("<UNK>"))?;
    // loop {
    //     let config = GetConfirmedSignaturesForAddress2Config {
    //         before: before.clone(),
    //         limit: Some(1000), // 最大1000，可分页
    //         ..Default::default()
    //     };
    //     let sigs = rpc
    //         .get_signatures_for_address_with_config(&PUBKEY_REDPACKET.parse()?, config)
    //         .await?;
    //
    //     if sigs.is_empty() {
    //         break;
    //     }
    //     for info in &sigs {
    //         if processed.contains(&info.signature) {
    //             continue;
    //         }
    //         processed.insert(info.signature.clone());
    //         // 拉取详细交易内容
    //         if let Ok(Some(tx)) = rpc
    //             .get_transaction(
    //                 &info.signature,
    //                 solana_transaction_status::UiTransactionEncoding::Json,
    //             )
    //             .await
    //         {
    //             if let Some(meta) = tx.transaction.meta {
    //                 if let Some(logs) = meta.log_messages {
    //                     println!("补拉日志：{:?}", logs);
    //                 }
    //             }
    //         }
    //     }
    //     // 下一页
    //     before = sigs.last().map(|x| x.signature.clone());
    //     if sigs.len() < 1000 {
    //         break;
    //     }
    // }
    Ok(())
}
