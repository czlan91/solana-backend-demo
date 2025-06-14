#![allow(clippy::too_many_arguments)]

use crate::red_packet::utils::Event;
use anchor_client::solana_client::rpc_config::RpcTransactionLogsConfig;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::Cluster;
use anchor_lang::__private::base64::prelude::BASE64_STANDARD;
use anchor_lang::__private::base64::Engine;
use anchor_lang::declare_program;
use anyhow::{Context, Result};
use solana_client::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcTransactionLogsFilter;
use std::time::Duration;
use tokio::time::sleep;

const PROGRAM_DATA: &str = "Program data: ";
declare_program!(red_packet);
const RECONNECT_DELAY: Duration = Duration::from_secs(3);

#[tokio::main]
async fn main() -> Result<()> {
    let cluster = Cluster::Devnet;

    loop {
        match run_subscription(&cluster).await {
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

async fn run_subscription(cluster: &Cluster) -> Result<()> {
    let ws_url = cluster.ws_url();
    let filter = RpcTransactionLogsFilter::Mentions(vec![
        "HqSDjxnoR35q8uRMG3LDDvbJ9Hqj4H4bWMPAsBF1hqJq".to_string(),
    ]);

    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::confirmed()),
    };

    let (_subscription, receiver) =
        PubsubClient::logs_subscribe(ws_url, filter, config).context("创建 WebSocket 订阅失败")?;

    println!("✅ 已连接至 {}，监听日志中...", ws_url);

    // 使用 match 显式处理 None 和 Some
    loop {
        match receiver.recv_timeout(Duration::from_secs(24 * 60 * 60)) {
            Ok(message) => {
                let logs = message.value.logs;

                for log in logs {
                    if let Some(data_str) = log.strip_prefix(PROGRAM_DATA) {
                        match BASE64_STANDARD.decode(data_str) {
                            Ok(decoded_data) => {
                                println!("📥 解码日志 (hex): {:02X?}", decoded_data);

                                // 事件解析
                                match Event::try_from_bytes(&decoded_data) {
                                    Ok(Event::RedPacketCreated(e)) => {
                                        println!("🎁 RedPacketCreated: {:?}", e);
                                    }
                                    Ok(Event::RedPacketClaimed(e)) => {
                                        println!("🎉 RedPacketClaimed: {:?}", e);
                                    }
                                    _ => {
                                        println!("⚠️ 未知事件或解析失败");
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("❌ base64 解码失败: {}", err);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // 如果 receiver 返回 None，代表 WebSocket 连接断开
                eprintln!("❌ WebSocket连接断开,{:?}", e);
                break; // 退出循环，进行重连或其他操作
            }
        }
    }
    // 如果接收到 None，代表连接关闭
    Err(anyhow::anyhow!("WebSocket连接断开"))
}
