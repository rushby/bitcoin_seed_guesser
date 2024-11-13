mod seed_generator;
mod balance_checker;
mod csv_logger;

use tokio::task;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::io;
use tokio::signal;
use crate::balance_checker::load_rich_list;
use tokio::sync::RwLock;
use std::collections::HashSet;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Load rich list into memory
    let rich_list_path = "rich_list.txt";
    let address_set = load_rich_list(rich_list_path).await.expect("Failed to load rich list");

    // Show the first 10 entries from the rich list
    {
        let address_set_read = address_set.read().await;
        println!("Top 10 rich list addresses (for reference):");
        for address in address_set_read.iter().take(10) {
            println!("Address: {}", address);
        }
        println!("---\n");
    }

    // Get user input for the word count (12 or 24)
    let word_count = get_user_word_count();

    // Cancellation flag to control process
    let is_running = Arc::new(AtomicBool::new(true));

    // Launch the guessing process in parallel, passing address_set
    let handle = tokio::spawn(start_guessing_process(word_count, address_set, is_running.clone()));

    // Wait for user cancellation (Ctrl+C)
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    println!("\nCanceling... Please wait for graceful shutdown.");

    // Stop the process and wait for the tasks to finish
    is_running.store(false, Ordering::SeqCst);
    handle.await.unwrap();
}

fn get_user_word_count() -> u8 {
    println!("Enter the number of words for the mnemonic (12 or 24): ");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    match input.trim() {
        "12" => 12,
        "24" => 24,
        _ => panic!("Invalid input. Use 12 or 24."),
    }
}

async fn start_guessing_process(
    word_count: u8, 
    address_set: Arc<RwLock<HashSet<String>>>, 
    is_running: Arc<AtomicBool>
) {
    let mut tasks = Vec::new();
    let counter = Arc::new(AtomicUsize::new(1)); // Unique ID counter

    for _ in 0..100 {  // Adjust to control parallelism
        let address_set = address_set.clone();
        let is_running = is_running.clone();
        let counter = counter.clone();
        
        let task = task::spawn(async move {
            while is_running.load(Ordering::SeqCst) {
                let id = counter.fetch_add(1, Ordering::SeqCst);
                let mnemonic = seed_generator::generate_seed_phrase(word_count);
                println!("\n[Seed ID: {}] Generated seed phrase: {}\n", id, mnemonic);

                for address_type in &[seed_generator::AddressType::P2PKH, seed_generator::AddressType::P2SH_P2WPKH, seed_generator::AddressType::Bech32] {
                    if let Ok(address) = seed_generator::derive_address(&mnemonic, *address_type) {
                        println!("[Seed ID: {}] Derived address ({:?}): {}", id, address_type, address);
                        
                        if balance_checker::check_address_exists(&address, &address_set).await {
                            println!("[Seed ID: {}] Address {} found in rich list!\n", id, address);
                            csv_logger::write_to_csv(&mnemonic.to_string(), &address).unwrap();
                            
                            // Stop the process by setting `is_running` to false
                            is_running.store(false, Ordering::SeqCst);
                            return;  // Exit the current task immediately
                        }
                    }
                }

                // Small delay to check for cancellation
                sleep(Duration::from_millis(10)).await;
            }
            println!("Task ending gracefully.");
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }
}
