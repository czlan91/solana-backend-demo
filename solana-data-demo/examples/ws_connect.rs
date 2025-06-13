use futures_util::StreamExt;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anyhow::{Context, Result};
use std::time::Duration;
use anchor_lang::__private::base64::Engine;
use anchor_lang::__private::base64::prelude::BASE64_STANDARD;
use anchor_lang::declare_program;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use tokio::time::sleep;

const PROGRAM_DATA: &str = "Program data: ";
const RECONNECT_DELAY: Duration = Duration::from_secs(3);
declare_program!(red_packet);
use crate::red_packet::utils::Event;
#[tokio::main]
async fn main() -> Result<()> {
    let ws_url = "wss://api.devnet.solana.com/"; // 或你自己的 cluster.ws_url()
    loop {
        match run_subscription(ws_url).await {
            Ok(_) => break,
            Err(e) => {
                eprintln!("连接中断: {:?}, {} 秒后重连...", e, RECONNECT_DELAY.as_secs());
                sleep(RECONNECT_DELAY).await;
            }
        }
    }
    Ok(())
}

async fn run_subscription(ws_url: &str) -> Result<()> {
    let filter = RpcTransactionLogsFilter::Mentions(vec![
        "HqSDjxnoR35q8uRMG3LDDvbJ9Hqj4H4bWMPAsBF1hqJq".to_string(),
    ]);
    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::confirmed()),
    };
    let client = PubsubClient::new(ws_url).await?;
    
    let (mut stream, _unsubscribe) =
        client.logs_subscribe(filter, config).await
            .context("创建 WebSocket 订阅失败")?;

    println!("✅ 已连接至 {}，监听日志中...", ws_url);

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