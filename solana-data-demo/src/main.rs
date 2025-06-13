use std::time::Duration;
use anchor_client::solana_client::rpc_config::RpcTransactionLogsConfig;
use anchor_client::{solana_sdk::commitment_config::CommitmentConfig, Cluster};
use anchor_lang::__private::base64::Engine;
use anchor_lang::__private::base64::prelude::BASE64_STANDARD;
use anchor_lang::declare_program;
use anyhow::Context;
use solana_client::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcTransactionLogsFilter;
use tokio::time::sleep;
use crate::red_packet::utils::Event;

const PROGRAM_DATA: &'static str = "Program data: ";
declare_program!(red_packet);
const RECONNECT_DELAY: Duration = Duration::from_secs(3); // 重连延迟

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cluster = Cluster::Devnet;

    // 持续重连循环
    loop {
        match run_subscription(&cluster).await {
            Ok(_) => break, // 正常退出
            Err(e) => {
                eprintln!("连接中断: {:?}, {}秒后重连...", e, RECONNECT_DELAY.as_secs());
                sleep(RECONNECT_DELAY).await;
            }
        }
    }
    Ok(())
}

async fn run_subscription(cluster: &Cluster) -> anyhow::Result<()> {
    let ws_url = cluster.ws_url();
    let filter = RpcTransactionLogsFilter::Mentions(vec!["HqSDjxnoR35q8uRMG3LDDvbJ9Hqj4H4bWMPAsBF1hqJq".to_string()]);

    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::confirmed()),
    };

    // 创建新连接
    let (subscription,  receiver) = PubsubClient::logs_subscribe(ws_url, filter, config)
        .context("创建WebSocket订阅失败")?;

    println!("成功连接到: {}", ws_url);

    while let response = receiver.recv()? {
        let logs = response.value.logs;

        for log in logs {
            if let Some(data_str) = log.strip_prefix(PROGRAM_DATA) {
                if let Ok(decoded_data) = BASE64_STANDARD.decode(data_str) {
                    println!("Decoded data (hex): {:?}", &decoded_data);

                    // 事件解析逻辑（根据你的实际事件结构修改）
                    match Event::try_from_bytes(&decoded_data) {
                        Ok(Event::RedPacketCreated(my_event)) =>
                            println!("RedPacketCreated Event: {:?}", my_event),
                        Ok(Event::RedPacketClaimed(my_event)) =>
                            println!("RedPacketClaimed Event: {:?}", my_event),
                        _ => println!("未知事件类型"),
                    }
                }
            }
        }
    }

    // 当receiver返回None时表示连接已断开
    anyhow::bail!("WebSocket连接意外关闭")
}
