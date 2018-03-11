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

fn put_record<T>(db: &DB, key: String, value: Vec<T>, writeopts: &rocksdb::WriteOptions)
    where T: serde::Serialize {
    db.put_opt(
        key.as_bytes(),
        serde_json::to_string(&value).unwrap().as_bytes(),
        writeopts);
}
fn no_prev(key: &str) -> bool {
    // Predicate of whether there is no previous for this transaction
    return key == "00000000000000000000000000000000000000000000000000000000000000004294967295";
}
fn extend_v<T>(mut present_v: Vec<T>, new_entry: T, key: &str) -> Vec<T>
    where T: Ord + PartialEq {
    if no_prev(key) {
        // In this case they are all unique (no duplicates), and it is a really big list
        return present_v;
    } else {
        present_v.push(new_entry);  // add new value
        present_v.sort();           // sort, to enable removing duplicates
        present_v.dedup();          // remove duplicates
        return present_v;
    }
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
        for result in rdr.deserialize() {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here..
            let record: TxIn = result?;         // parse CSV
            let new_entry = TxInSm {            // Convert to smaller type
                txid: record.txid,
            };
            // Check what we already have stored
            let key: String = format!("{}{}", &record.hashprevout, &record.indexprevout);
            let tx_v: Vec<TxInSm> = get_present_in(&db, key.clone());
            put_record(&db, key.clone(), extend_v(tx_v, new_entry, &key), &write_options);
        }
    } else {
        for result in rdr.deserialize() {
            let record: TxOut = result?;        // parse CSV
            let new_entry = TxOutSm {           // Convert to smaller type
                indexout: record.indexout,
                value: record.value,
                address: record.address
            };
            // Check what we already have stored
            let key: String = record.txid.clone();
            let tx_v: Vec<TxOutSm> = get_present_out(&db, key.clone());
            put_record(&db, key.clone(), extend_v(tx_v, new_entry, &key), &write_options);
        }
    }
        // let rand_n = rand::thread_rng().gen_range(1, 1001);
        // if rand_n > 990 { // so that printing doesn't become the bottleneck
        //     let check = get_present(&db, record.txid.clone());
        //     println!("{}", serde_json::to_string(&check).unwrap());
        // }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "tx_in" {
        if let Err(err) = save_tx("tx_in") {
            println!("error running save_tx: {}", err);
            process::exit(1);
        }
    } else if args[1] == "tx_out" {
        if let Err(err) = save_tx("tx_out") {
            println!("error running save_tx_out: {}", err);
            process::exit(1);
        }
    }
}

// cargo run tx_out < ../csv-data/tx_out_sample.csv
// cargo run tx_in < ../csv-data/tx_in_sample.csv
