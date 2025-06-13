use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey},
    Cluster,
};
use anchor_lang::{declare_program};
use std::str::FromStr;
use crate::red_packet::utils::Account;

declare_program!(red_packet);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let rpc_url = "https://api.mainnet-beta.solana.com"; // 主网
    let rpc_url = Cluster::Devnet.url();
    let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let pubkey = Pubkey::from_str("AZJpvXzbVt91fhUnTsD8DewKepKTUh7CpoBy1TSqmgq2")?;
    let account = rpc_client.get_account(&pubkey)?;

    match Account::try_from_bytes(&account.data) {
        Ok(Account::RedPacket(red_packet_data)) => {
            println!("RedPacket data: {:?}", red_packet_data)
        }
        Err(e) => return Err(e.into()),
        _ => {
            println!("Not a valid event")
        }
    }
    Ok(())
}
