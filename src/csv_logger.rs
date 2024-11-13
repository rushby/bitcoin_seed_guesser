use std::fs::OpenOptions;
use csv::WriterBuilder;

pub fn write_to_csv(seed_phrase: &str, address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = OpenOptions::new().create(true).append(true).open("found_addresses.csv")?;
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);
    wtr.write_record(&[seed_phrase, address])?;
    wtr.flush()?;
    Ok(())
}
