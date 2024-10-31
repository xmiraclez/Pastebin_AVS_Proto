#![allow(missing_docs)]

use alloy_primitives::{eip191_hash_message, Address, FixedBytes, U256};
use alloy_provider::Provider;
use alloy_rpc_types::{BlockNumberOrTag, Filter};
use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{sol, SolEvent};
use chrono::Utc;
use dotenv::dotenv;
use eigen_client_elcontracts::{
    reader::ELChainReader,
    writer::{ELChainWriter, Operator},
};
use eigen_logging::{get_logger, init_logger, log_level::LogLevel, logger::Logger, EigenLogger};
use eigen_utils::get_signer;
use eyre::Result;
use once_cell::sync::Lazy;
use rand::RngCore;
use std::{env, str::FromStr};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    HelloWorldServiceManager,
    "json_abi/HelloWorldServiceManager.json"
);

static KEY: Lazy<String> = Lazy::new(|| env::var("HOLESKY_PRIVATE_KEY").expect("failed to retrieve private key"));
pub static RPC_URL: Lazy<String> = Lazy::new(|| env::var("HOLESKY_RPC_URL").expect("failed to get rpc url from env"));
pub static HELLO_WORLD_CONTRACT_ADDRESS: Lazy<String> = Lazy::new(|| {
    env::var("HOLESKY_CONTRACT_ADDRESS").expect("failed to get hello world contract address from env")
});

static DELEGATION_MANAGER_CONTRACT_ADDRESS: Lazy<String> = Lazy::new(|| {
    env::var("HOLESKY_DELEGATION_MANAGER_ADDRESS").expect("failed to get delegation manager contract address from env")
});

static STAKE_REGISTRY_CONTRACT_ADDRESS: Lazy<String> = Lazy::new(|| {
    env::var("HOLESKY_STAKE_REGISTRY_ADDRESS").expect("failed to get stake registry contract address from env")
});

static AVS_DIRECTORY_CONTRACT_ADDRESS: Lazy<String> = Lazy::new(|| {
    env::var("HOLESKY_AVS_DIRECTORY_ADDRESS").expect("failed to get avs directory contract address from env")
});

// Функция валидации контента пасты
fn validate_paste_content(content: &str) -> (bool, String) {
    // Проверка размера
    if content.len() > 10000 {
        return (false, "Content too large".to_string());
    }

    // Проверка на пустой контент
    if content.trim().is_empty() {
        return (false, "Content is empty".to_string());
    }

    // Проверка на спам (простой пример)
    let spam_words = ["spam", "scam", "hack"];
    for word in spam_words.iter() {
        if content.to_lowercase().contains(word) {
            return (false, format!("Content contains forbidden word: {}", word));
        }
    }

    (true, "Content is valid".to_string())
}

async fn sign_and_validate_paste(
    paste_id: U256,
    creator: Address,
    content: String,
    timestamp: U256,
) -> Result<()> {
    println!("Processing paste event:");
    println!("ID: {}", paste_id);
    println!("Creator: {}", creator);
    println!("Content: {}", content);
    println!("Timestamp: {}", timestamp);

    // Валидация контента
    let (is_valid, reason) = validate_paste_content(&content);
    println!("Validation result: {}, Reason: {}", is_valid, reason);

    // Подписываем результат валидации
    let provider = get_signer(KEY.clone(), &RPC_URL);
    let msg_hash = eip191_hash_message(content.clone());
    let wallet = PrivateKeySigner::from_str(&KEY.clone()).expect("failed to generate wallet");
    let signature = wallet.sign_hash_sync(&msg_hash)?;

    let hello_world_contract_address = Address::from_str(&HELLO_WORLD_CONTRACT_ADDRESS)
        .expect("wrong hello world contract address");
    let hello_world_contract =
        HelloWorldServiceManager::new(hello_world_contract_address, &provider);

    // Отправляем результат валидации в смарт-контракт
    hello_world_contract
        .validatePaste(paste_id, is_valid, reason, signature.as_bytes().into())
        .send()
        .await?
        .get_receipt()
        .await?;

    println!("Paste validation submitted");
    Ok(())
}

async fn sign_and_response_to_task(
    task_index: u32,
    task_created_block: u32,
    name: String,
) -> Result<()> {
    let provider = get_signer(KEY.clone(), &RPC_URL);
    let message = format!("Hello, {}", name);
    let msg_hash = eip191_hash_message(message);
    let wallet = PrivateKeySigner::from_str(&KEY.clone()).expect("failed to generate wallet");
    let signature = wallet.sign_hash_sync(&msg_hash)?;

    println!("Signing and responding to task : {:?}", task_index);

    let hello_world_contract_address = Address::from_str(&HELLO_WORLD_CONTRACT_ADDRESS)
        .expect("wrong hello world contract address");
    let hello_world_contract =
        HelloWorldServiceManager::new(hello_world_contract_address, &provider);

    hello_world_contract
        .respondToTask(
            HelloWorldServiceManager::Task {
                name,
                taskCreatedBlock: task_created_block,
            },
            task_index,
            signature.as_bytes().into(),
        )
        .send()
        .await?
        .get_receipt()
        .await?;

    println!("Responded to task");
    Ok(())
}

async fn monitor_new_tasks() -> Result<()> {
    let provider = get_signer(KEY.clone(), &RPC_URL);
    let hello_world_contract_address = Address::from_str(&HELLO_WORLD_CONTRACT_ADDRESS)
        .expect("wrong hello world contract address");
    let hello_world_contract =
        HelloWorldServiceManager::new(hello_world_contract_address, &provider);

    // Получаем текущий блок и начинаем с блока на 10 назад
    let current_block = provider.get_block_number().await?;
    let mut latest_processed_block = current_block.saturating_sub(10);

    loop {
        println!("Monitoring for new tasks and pastes...");

        let filter = Filter::new()
            .address(hello_world_contract_address)
            .from_block(BlockNumberOrTag::Number(latest_processed_block));

        let logs = provider.get_logs(&filter).await?;

        for log in logs {
            match log.topic0() {
                Some(&HelloWorldServiceManager::NewTaskCreated::SIGNATURE_HASH) => {
                    let HelloWorldServiceManager::NewTaskCreated { taskIndex, task } = log
                        .log_decode()
                        .expect("Failed to decode log new task created")
                        .inner
                        .data;

                    println!("New task detected :Hello{:?} ", task.name);
                    let _ = sign_and_response_to_task(taskIndex, task.taskCreatedBlock, task.name)
                        .await;
                }
                Some(&HelloWorldServiceManager::PasteCreated::SIGNATURE_HASH) => {
                    let HelloWorldServiceManager::PasteCreated { id, creator, content, timestamp } = log
                        .log_decode()
                        .expect("Failed to decode paste created event")
                        .inner
                        .data;

                    println!("New paste detected: ID {}", id);
                    let _ = sign_and_validate_paste(id, creator, content, timestamp).await;
                }
                _ => {}
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
        let current_block = provider.get_block_number().await?;
        latest_processed_block = current_block;
    }
}

async fn register_operator() -> Result<()> {
    let wallet = PrivateKeySigner::from_str(&KEY).expect("failed to generate wallet");
    let provider = get_signer(KEY.clone(), &RPC_URL);

    let delegation_manager_contract_address = Address::from_str(&DELEGATION_MANAGER_CONTRACT_ADDRESS)
        .expect("wrong delegation manager contract address");

    println!(
        "delegation manager :{}",
        delegation_manager_contract_address
    );

    let avs_directory_contract_address = Address::from_str(&AVS_DIRECTORY_CONTRACT_ADDRESS)
        .expect("wrong avs directory contract address");

    let stake_registry_contract_address = Address::from_str(&STAKE_REGISTRY_CONTRACT_ADDRESS)
        .expect("wrong stake registry contract address");

    let default_slasher = Address::ZERO;
    let default_strategy = Address::ZERO;

    let elcontracts_reader_instance = ELChainReader::new(
        get_logger(),
        default_slasher,
        delegation_manager_contract_address,
        avs_directory_contract_address,
        RPC_URL.clone(),
    );

    let elcontracts_writer_instance = ELChainWriter::new(
        delegation_manager_contract_address,
        default_strategy,
        elcontracts_reader_instance.clone(),
        RPC_URL.clone(),
        KEY.clone(),
    );

    let operator = Operator::new(
        wallet.address(),
        wallet.address(),
        Address::ZERO,
        0u32,
        None,
    );

    let _tx_hash = elcontracts_writer_instance.register_as_operator(operator).await;

    println!("Operator registered on EL successfully");
    println!(
        "Operator registered on AVS successfully :{}",
        wallet.address()
    );

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logger(LogLevel::Info);

    //if let Err(e) = register_operator().await {
        //eprintln!("Failed to register operator: {:?}", e);
    //    return;
    //}

    if let Err(e) = monitor_new_tasks().await {
        eprintln!("Failed to monitor new tasks: {:?}", e);
    }
}
