#![allow(missing_docs)]

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use dotenv::dotenv;
use eigen_utils::get_signer;
use eyre::Result;
use std::{env, str::FromStr};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "paste_reader", about = "Read pastes from EigenLayer AVS Pastebin")]
struct Opt {
    /// ID of the paste to read
    #[structopt(name = "PASTE_ID")]
    paste_id: u64,
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    HelloWorldServiceManager,
    "json_abi/HelloWorldServiceManager.json"
);

async fn read_paste(paste_id: u64) -> Result<()> {
    dotenv().ok();
    
    let rpc_url = env::var("HOLESKY_RPC_URL")
        .expect("failed to get rpc url from env");
    let contract_address = env::var("HOLESKY_CONTRACT_ADDRESS")
        .expect("failed to get contract address from env");

    // Создаем пустой приватный ключ для read-only операций
    let empty_key = "1111111111111111111111111111111111111111111111111111111111111111".to_string();
    
    // Инициализируем провайдера
    let provider = get_signer(empty_key, &rpc_url);
    let contract_address = Address::from_str(&contract_address)
        .expect("wrong contract address");
    let contract = HelloWorldServiceManager::new(contract_address, &provider);

    // Преобразуем paste_id в U256
    let paste_id_u256 = U256::from(paste_id);

    println!("Fetching paste #{}", paste_id);

    // Получаем пасту
    let paste = contract
        .getPaste(paste_id_u256)
        .call()
        .await?;

    // Выводим информацию о пасте
    println!("\nPaste Details:");
    println!("ID: {}", paste.pasteId);
    println!("Creator: {}", paste.creator);
    println!("Timestamp: {}", paste.timestamp);
    println!("Validated: {}", paste.isValidated);
    println!("Validations Count: {}", paste.validationsCount);
    println!("Published: {}", paste.isPublished);
    println!("\nContent:");
    println!("{}", paste.content);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    
    println!("Reading paste #{}...", opt.paste_id);
    
    if let Err(e) = read_paste(opt.paste_id).await {
        eprintln!("Error reading paste: {:?}", e);
    }

    Ok(())
}
