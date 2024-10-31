#![allow(missing_docs)]

use alloy_primitives::Address;
use alloy_sol_types::{sol, SolEvent};
use dotenv::dotenv;
use eigen_utils::get_signer;
use eyre::Result;
use std::{env, str::FromStr};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "paste_client", about = "Create pastes in EigenLayer AVS Pastebin")]
struct Opt {
    /// Text content for the paste
    #[structopt(name = "CONTENT")]
    content: Vec<String>,
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    HelloWorldServiceManager,
    "json_abi/HelloWorldServiceManager.json"
);

async fn create_paste(content: &str) -> Result<()> {
    dotenv().ok();
    
    let rpc_url = env::var("HOLESKY_RPC_URL")
        .expect("failed to get rpc url from env");
    let contract_address = env::var("HOLESKY_CONTRACT_ADDRESS")
        .expect("failed to get contract address from env");
    let private_key = env::var("HOLESKY_PRIVATE_KEY")
        .expect("failed to get private key from env");

    let provider = get_signer(private_key, &rpc_url);
    let contract_address = Address::from_str(&contract_address)
        .expect("wrong contract address");
    let contract = HelloWorldServiceManager::new(contract_address, &provider);

    println!("\nCreating paste with content:\n{}", content);
    
    let tx_result = contract
        .createPaste(content.to_string())
        .send()
        .await?
        .get_receipt()
        .await?;

    println!("\nPaste created successfully!");
    println!("Transaction hash: {:?}", tx_result.transaction_hash);

    for log in tx_result.inner.logs() {
        if let Some(topic0) = log.topic0() {
            if topic0 == &HelloWorldServiceManager::PasteCreated::SIGNATURE_HASH {
                if let Ok(event) = log.log_decode::<HelloWorldServiceManager::PasteCreated>() {
                    println!("\nPaste details:");
                    println!("ID: {}", event.inner.data.id);
                    println!("Creator: {}", event.inner.data.creator);
                    println!("Timestamp: {}", event.inner.data.timestamp);
                    break;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    
    let content = opt.content.join(" ");
    
    if content.is_empty() {
        println!("Usage: paste_client <your text here>");
        println!("Example: paste_client This is my first paste");
        return Ok(());
    }

    if let Err(e) = create_paste(&content).await {
        eprintln!("Error creating paste: {:?}", e);
    }

    Ok(())
}
