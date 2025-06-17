use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey},
    Cluster,
};
use anchor_lang::declare_program;
use borsh::BorshDeserialize;
use spl_token_2022::extension::metadata_pointer::MetadataPointer;
use spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};
use spl_token_2022::state::Mint;
use spl_token_metadata_interface::state::TokenMetadata;
use std::str::FromStr;

declare_program!(red_packet);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let rpc_url = "https://api.mainnet-beta.solana.com"; // 主网
    let rpc_url = Cluster::Devnet.url();
    let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let pubkey = Pubkey::from_str("3QvdZgaBcoqeDrPWVGFaHmo2KCyQfKVQc5J3ygnt3C9M")?;
    let account = rpc_client.get_account(&pubkey)?;
    println!("Account data: {:?}", account);
    let state = StateWithExtensions::<Mint>::unpack(&account.data)?;

    println!("=== Mint === {:?}", state);
    // 解析 MetadataPointer
    match state.get_extension::<MetadataPointer>() {
        Ok(pointer) => {
            println!("\n=== MetadataPointer ===");
            println!("元数据权限: {:?}", pointer.authority);
            println!("元数据地址: {:?}", pointer.metadata_address);
        }
        Err(_) => println!("\nMetadataPointer 扩展不存在"),
    }

    // 解析 TokenMetadata
    // --- 这里是修复的关键部分 ---
    // 解析 TokenMetadata (非 Pod 类型)
    // 1. 使用 get_extension_bytes 获取原始字节
    // 2. 使用 TokenMetadata::unpack 手动解析字节
    match state.get_extension_bytes::<TokenMetadata>() {
        Ok(bytes) => {
            let metadata = TokenMetadata::try_from_slice(bytes)?;
            println!("\n=== TokenMetadata (使用 get_extension_bytes + from_bytes) ===");
            println!("更新权限: {:?}", metadata.update_authority);
            println!("Mint: {:?}", metadata.mint);
            println!("名称: {}", metadata.name);
            println!("符号: {}", metadata.symbol);
            println!("URI: {}", metadata.uri);
            println!("额外元数据:");
            for (key, value) in metadata.additional_metadata {
                println!("  - {}: {}", key, value);
            }
        }
        Err(_) => println!("\nTokenMetadata 扩展不存在"),
    }

    Ok(())
}
