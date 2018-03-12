extern crate rand;
extern crate csv;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate rocksdb;

// For Reading from stdin
use std::error::Error;
use std::io;
use std::process;
use std::str;       // strings

use rand::Rng;      // random number generation
use rocksdb::DB;    // working with the rocksdb
use std::env;       // command line arguments

const TX_OUT_DB: &'static str = "../graph_data/tx_out_rdb";
const TX_IN_DB: &'static str = "../graph_data/tx_in_rdb";
const START_HASH_IDX: &'static str = "00000000000000000000000000000000000000000000000000000000000000004294967295";

#[derive(Debug, Deserialize)]
struct TxIn {
    txid: String,
    hashprevout: String,
    indexprevout: String,
    scriptsig: String,
    sequence: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct TxInSm {
    txid: String,
}

#[derive(Debug, Deserialize)]
struct TxOut {
    txid: String,
    indexout: String,
    value: String,
    scriptpubkey: String,
    address: String,
    unspent: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct TxOutSm {
    indexout: String,
    value: String,
    address: String,
}

// Database operations
fn get_present_out(db: &DB, key: String) -> Vec<TxOutSm> {
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => serde_json::from_slice(&value[..]).unwrap(),
        Ok(None) => Vec::new(), // empty vector
        Err(e) => panic!("{}", e)
    }
}
fn get_present_in(db: &DB, key: String) -> Vec<TxInSm> {
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => serde_json::from_slice(&value[..]).unwrap(),
        Ok(None) => Vec::new(), // empty vector
        Err(e) => panic!("{}", e)
    }
}
fn update_entry_in(db: &DB, key: String, new_entry: TxInSm, writeopts: &rocksdb::WriteOptions) {
    // Update the entry, rather than overwriting it
    let present_v: Vec<TxInSm> = get_present_in(&db, key.clone());
    put_record(&db, key.clone(), extend_v(present_v, new_entry), writeopts);
}
fn put_record<T>(db: &DB, key: String, value: Vec<T>, writeopts: &rocksdb::WriteOptions)
    where T: serde::Serialize {
    db.put_opt(
        key.as_bytes(),
        serde_json::to_string(&value).unwrap().as_bytes(),
        writeopts);
}
fn get_start_txid(db: &DB) -> Vec<TxInSm> {
    println!("{}", String::from(START_HASH_IDX));
    return get_present_in(db, String::from(START_HASH_IDX));
}

fn no_prev(key: &str) -> bool {
    // Predicate of whether there is no previous for this transaction
    return key == START_HASH_IDX;
}
fn extend_v<T>(mut present_v: Vec<T>, new_entry: T) -> Vec<T>
    where T: Ord + PartialEq {
    present_v.push(new_entry);  // add new value
    present_v.sort();           // sort, to enable removing duplicates
    present_v.dedup();          // remove duplicates
    return present_v;
}

fn save_tx(direction: &str) -> Result<(), Box<Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    // Connect to the rocksdb database
    let db_location: &str = match direction {
        "tx_in" => TX_IN_DB,
        _ => TX_OUT_DB,
    };
    let db = DB::open_default(db_location).unwrap();
    let mut write_options = rocksdb::WriteOptions::default();
    // write_options.set_sync(false);
    write_options.disable_wal(true);
    if direction == "tx_in" {
        let mut start_v: Vec<TxInSm> = Vec::new();
        for result in rdr.deserialize() {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here..
            let record: TxIn = result?;         // parse CSV
            let new_entry = TxInSm {            // Convert to smaller type
                txid: record.txid,
            };
            let key: String = format!("{}{}", &record.hashprevout, &record.indexprevout);
            if !no_prev(&key) { 
                update_entry_in(&db, key.clone(), new_entry, &write_options);
            } else {
                // There are lots of transactions with no previous transactions.
                // This allows building a buffer and saving them all at once.
                start_v.push(new_entry);
            }
        }
        // Save the ones without previous transactions
        let mut present_v: Vec<TxInSm> = get_present_in(
            &db, String::from(START_HASH_IDX));         // Get present vector from db
        present_v.append(&mut start_v);                 // Add buffered vector
        present_v.sort();                               // Prep for dedup
        present_v.dedup();                              // Remote duplicates
        put_record(                                     // Save to db
            &db, String::from(START_HASH_IDX), 
            present_v, &write_options); 
    } else {
        for result in rdr.deserialize() {
            let record: TxOut = result?;                // parse CSV
            let new_entry = TxOutSm {                   // Convert to smaller type
                indexout: record.indexout,
                value: record.value,
                address: record.address
            };
            let key: String = record.txid.clone();
            let tx_v: Vec<TxOutSm> = get_present_out(&db, key.clone());
            put_record(&db, key.clone(), extend_v(tx_v, new_entry), &write_options);
        }
    }
        // let rand_n = rand::thread_rng().gen_range(1, 1001);
        // if rand_n > 990 { // so that printing doesn't become the bottleneck
        //     let check = get_present(&db, record.txid.clone());
        //     println!("{}", serde_json::to_string(&check).unwrap());
        // }
    Ok(())
}

fn random_walk() {
    // -> Result<(), Box<Error>> {
    // Randomly walk the stored graph
    //
    // connect to both dbs
    let db_out = DB::open_default(TX_OUT_DB).unwrap();
    let db_in = DB::open_default(TX_IN_DB).unwrap();
    let st = get_start_txid(&db_in);
    println!("{:?}", st.len())
    // pick a random tx_id (could always start the base one "0000...")
    // get_present_out(txid)
        // txid -> (txhash, index, address) # query tx_out
        //     Lookup the txid in tx_out
        //     Get the set of outputs
        //     Choose an ouput (weighted randomization by amount)
        //     Return the address that recieved that output
    // get_present_in(format!("{}{}", txhash, index))
        // (txhash, index) -> tx_id # query tx_in
        //     Find next transacion
        //     Find a transaction that has that txhash and index as a previous output (input)
        //     Return the id for that transaction
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "tx_in" { // import tx_in
        if let Err(err) = save_tx("tx_in") {    
            println!("error running save_tx: {}", err);
            process::exit(1);
        }
    } else if args.len() > 1 && args[1] == "tx_out" { // import tx_out
        if let Err(err) = save_tx("tx_out") {
            println!("error running save_tx_out: {}", err);
            process::exit(1);
        }
    } else { // random walk
        random_walk();
    }
}

// cargo run tx_out < ../csv-data/tx_out_sample.csv
// cargo run tx_in < ../csv-data/tx_in_sample.csv
