use anchor_client::{
    Cluster,
    solana_client::rpc_client::RpcClient,
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey},
};
use anchor_lang::declare_program;
use std::str::FromStr;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};
use anchor_spl::token_2022::spl_token_2022::extension::metadata_pointer::MetadataPointer;
use anchor_spl::token_2022::spl_token_2022::extension::token_metadata::TokenMetadata;
use anchor_spl::token_2022::spl_token_2022::state::Mint;

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

    // 尝试解析 MetadataPointer
    match state.get_extension::<MetadataPointer>() {
        Ok(pointer) => {
            println!("\n=== MetadataPointer ===");
            println!("元数据权限: {:?}", pointer.authority);
            println!("元数据地址: {:?}", pointer.metadata_address);
        }
        Err(_) => println!("\nMetadataPointer 扩展不存在"),
    }

    // 尝试解析 TokenMetadata
    match state.get_extension::<TokenMetadata>() {
        Ok(metadata) => {
            println!("\n=== TokenMetadata ===");
            println!("名称: {}", metadata.name());
            println!("符号: {}", metadata.symbol());
            println!("URI: {}", metadata.uri());

            if !metadata.additional_metadata().is_empty() {
                println!("\n额外元数据:");
                for (key, value) in metadata.additional_metadata() {
                    println!("  {}: {}", key, value);
                }
            }
        }
        Err(_) => println!("\nTokenMetadata 扩展不存在"),
    }

    Ok(())
}
