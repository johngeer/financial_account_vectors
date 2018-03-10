extern crate rand;
extern crate csv;
#[macro_use]
extern crate serde_derive;
extern crate serde;

// For reading from stdin
use std::error::Error;
use std::io;
use std::process;

use rand::Rng;

#[derive(Debug,Deserialize)]
struct TxIn {
    txid: String,
    hashprevout: String,
    indexprevout: String,
    scriptsig: String,
    sequence: String,
}

#[derive(Debug,Deserialize)]
struct TxOut {
    txid: String,
    indexout: String,
    value: String,
    scriptpubkey: String,
    address: String,
    unspent: String,
}

fn example() -> Result<(), Box<Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here..
        let record: TxOut = result?;
        let rand_n = rand::thread_rng().gen_range(1, 1001);
        if rand_n > 999 { // so that printing doesn't become the bottleneck
            println!("{:?}", record.value);
        }
    }
    Ok(())
}

fn main() {
    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}

// cargo run < ../csv-data/tx_in_sample.csv
// cat ../csv-data/tx_in_sample.csv
