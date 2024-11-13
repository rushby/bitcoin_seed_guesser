use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::error::Error;

type AddressSet = Arc<RwLock<HashSet<String>>>;

pub async fn load_rich_list(file_path: &str) -> Result<AddressSet, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut address_set = HashSet::new();

    for line in reader.lines() {
        let line = line?;
        if let Some(address) = parse_address_from_line(&line) {
            address_set.insert(address);
        }
    }

    Ok(Arc::new(RwLock::new(address_set)))
}

fn parse_address_from_line(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }
    // Return only the address
    Some(parts[1].to_string())
}

pub async fn check_address_exists(address: &str, address_set: &AddressSet) -> bool {
    let set = address_set.read().await;
    set.contains(address)
}
