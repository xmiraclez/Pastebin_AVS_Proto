#![allow(missing_docs)]

use alloy_primitives::Address;
use alloy_sol_types::sol;
use dotenv::dotenv;
use eigen_logging::{get_logger, init_logger, log_level::LogLevel};
use eigen_utils::get_signer;
use eyre::Result;
use once_cell::sync::Lazy;
use rand::Rng;
use std::{env, str::FromStr};
use tokio::time::{self, Duration};
use chrono::Utc;

pub static RPC_URL: Lazy<String> = Lazy::new(|| {
    env::var("HOLESKY_RPC_URL").expect("failed to get rpc url from env")
});

pub static HELLO_WORLD_CONTRACT_ADDRESS: Lazy<String> = Lazy::new(|| {
    env::var("HOLESKY_CONTRACT_ADDRESS").expect("failed to get hello world contract address from env")
});

static KEY: Lazy<String> = Lazy::new(|| {
    env::var("HOLESKY_PRIVATE_KEY").expect("failed to retrieve private key")
});

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    HelloWorldServiceManager,
    "json_abi/HelloWorldServiceManager.json"
);

fn generate_paste_content() -> String {
    let contents = [
        "# Welcome to EigenLayer Paste\nThis is a test paste created by the AVS operator.",
        "## Smart Contract Testing\nTesting EigenLayer AVS Pastebin functionality.",
        "### Code Example\n```rust\nfn hello_world() {\n    println!(\"Hello, EigenLayer!\");\n}```",
        "#### Documentation\nThis is a sample documentation paste for testing purposes.",
        "##### Test Results\nAll tests passed successfully. System is working as expected."
    ];
    
    let mut rng = rand::thread_rng();
    let base_content = contents[rng.gen_range(0..contents.len())];
    let timestamp = Utc::now().timestamp();
    
    format!(
        "{}\n\nTimestamp: {}\nGenerated by: EigenLayer AVS Operator",
        base_content,
        timestamp
    )
}

async fn create_new_paste(content: &str) -> Result<()> {
    let hello_world_contract_address = Address::from_str(&HELLO_WORLD_CONTRACT_ADDRESS)
        .expect("wrong hello world contract address");
    let provider = get_signer(KEY.clone(), &RPC_URL);
    let hello_world_contract =
        HelloWorldServiceManager::new(hello_world_contract_address, &provider);

    let tx_result = hello_world_contract
        .createPaste(content.to_string())
        .send()
        .await?
        .get_receipt()
        .await?;

    println!("Paste created successfully!");
    println!("Transaction hash: {:?}", tx_result.transaction_hash);
    println!("Content preview: {}", &content[..content.len().min(100)]);

    Ok(())
}

fn generate_random_name() -> String {
    let adjectives = ["Quick", "Lazy", "Sleepy", "Noisy", "Hungry"];
    let nouns = ["Fox", "Dog", "Cat", "Mouse", "Bear"];
    let mut rng = rand::thread_rng();
    let adjective = adjectives[rng.gen_range(0..adjectives.len())];
    let noun = nouns[rng.gen_range(0..nouns.len())];
    let number: u16 = rng.gen_range(0..1000);
    format!("{}{}{}", adjective, noun, number)
}

async fn create_new_task(task_name: &str) -> Result<()> {
    let hello_world_contract_address = Address::from_str(&HELLO_WORLD_CONTRACT_ADDRESS)
        .expect("wrong hello world contract address");
    let provider = get_signer(KEY.clone(), &RPC_URL);
    let hello_world_contract =
        HelloWorldServiceManager::new(hello_world_contract_address, &provider);

    let tx = hello_world_contract
        .createNewTask(task_name.to_string())
        .send()
        .await?
        .get_receipt()
        .await?;

    println!(
        "Task created successfully with tx : {:?}",
        tx.transaction_hash
    );

    Ok(())
}

async fn start_creating_content() {
    let mut interval = time::interval(Duration::from_secs(15));
    init_logger(LogLevel::Info);

    loop {
        interval.tick().await;
        
        // Создаем пасту
        let paste_content = generate_paste_content();
        println!("\nCreating new paste...");
        if let Err(e) = create_new_paste(&paste_content).await {
            println!("Error creating paste: {:?}", e);
        }

        // Создаем обычную задачу
        let random_name = generate_random_name();
        println!("\nCreating new task with name: {}", random_name);
        if let Err(e) = create_new_task(&random_name).await {
            println!("Error creating task: {:?}", e);
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("Starting EigenLayer AVS Pastebin content generator...");
    start_creating_content().await;
}
