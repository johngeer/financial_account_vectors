extern crate rand;
extern crate csv;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate rocksdb;

// For reading from stdin
use std::error::Error;
use std::io;
use std::process;
use std::str;

use rand::Rng;
use rocksdb::DB;

const TX_OUT_DB: &'static str = "../graph_data/tx_out_rdb";
const TX_IN_DB: &'static str = "../graph_data/tx_in_rdb";

#[derive(Debug,Deserialize)]
struct TxIn {
    txid: String,
    hashprevout: String,
    indexprevout: String,
    scriptsig: String,
    sequence: String,
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

fn get_present(db: &DB, key: String) -> Vec<TxOutSm> {
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => serde_json::from_slice(&value[..]).unwrap(),
        Ok(None) => Vec::new(), // empty vector
        Err(e) => panic!("{}", e)
    }
}
fn put_record(db: &DB, key: String, value: Vec<TxOutSm>) {
    db.put(
        key.as_bytes(),
        serde_json::to_string(&value).unwrap().as_bytes());
}
fn extend_v<T>(mut present_v: Vec<T>, new_entry: T) -> Vec<T>
    where T: Ord + PartialEq {
    present_v.push(new_entry);  // add new value
    present_v.sort();           // sort, to enable removing duplicates
    present_v.dedup();          // remove duplicates
    return present_v;
}

fn save_tx_out() -> Result<(), Box<Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    // Connect to the rocksdb database
    let db = DB::open_default(TX_OUT_DB).unwrap();
    for result in rdr.deserialize() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here..
        let record: TxOut = result?;    // parse CSV
        let value = TxOutSm {           // Convert to smaller type
            indexout: record.indexout,
            value: record.value,
            address: record.address
        };
        // Check what we already have stored
        let tx_out_v = get_present(&db, record.txid.clone());
        put_record(&db, record.txid.clone(), extend_v(tx_out_v, value));

        // let rand_n = rand::thread_rng().gen_range(1, 1001);
        // if rand_n > 990 { // so that printing doesn't become the bottleneck
        //     let check = get_present(&db, record.txid.clone());
        //     println!("{}", serde_json::to_string(&check).unwrap());
        // }
    }
    Ok(())
}

fn main() {
    if let Err(err) = save_tx_out() {
        println!("error running save_tx_out: {}", err);
        process::exit(1);
    }
}

// cargo run < ../csv-data/tx_out_sample.csv
