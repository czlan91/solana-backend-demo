use anchor_client::Cluster;
use anchor_lang::prelude::Pubkey;
use anchor_lang::pubkey;
use mpl_token_metadata::accounts::Metadata;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

fn get_metadata_pda(mint_address: &Pubkey) -> Pubkey {
    let metadata_program_id = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
    let seeds = &[
        b"metadata".as_ref(),
        metadata_program_id.as_ref(),
        mint_address.as_ref(),
    ];
    let (pda, _bump) = Pubkey::find_program_address(seeds, &metadata_program_id);
    pda
}

// 使用示例
fn main() -> anyhow::Result<()> {
    let mint = pubkey!("So11111111111111111111111111111111111111112");
    let metadata_pda = get_metadata_pda(&mint);
    println!("Metadata PDA: {}", metadata_pda);

    let rpc_url = Cluster::Devnet.url();
    let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let account = rpc_client.get_account(&metadata_pda)?;

    println!("Account: {:?}", account);
    println!("\nParsing metadata...");
    // base64 解密
    match Metadata::from_bytes(&account.data) {
        Ok(metadata) => {
            println!("Metadata: {:?}", metadata);
            println!("\n=== On-Chain Metadata ===");
            println!("Key: {:?}", metadata.key);
            println!("Update Authority: {}", metadata.update_authority);
            println!("Mint: {}", metadata.mint);
            println!("Name: {}", metadata.name);
            println!("Symbol: {}", metadata.symbol);
            println!("URI: {}", metadata.uri);
            println!(
                "Seller Fee Basis Points: {}",
                metadata.seller_fee_basis_points
            );
            println!("Primary Sale Happened: {}", metadata.primary_sale_happened);
            println!("Is Mutable: {}", metadata.is_mutable);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    Ok(())
}
