use std::path::PathBuf;
use std::fs;

use bitcoin_hashes::{sha256d, sha256t_hash_newtype};
use bitcoin::{block::Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TxId(pub Vec<u8>);

#[derive(Debug, Serialize, Deserialize)]
struct Version(pub i32);

#[derive(Debug, Serialize, Deserialize)]
struct ScriptBuf(pub Vec<u8>);

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    /// The protocol version, is currently expected to be 1 or 2 (BIP 68).
    pub version: Version,
    /// List of transaction inputs
    pub vin: Vec<TxIn>,
    /// List of transaction outputs
    pub output: Vec<TxOut>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Amount(u64);

#[derive(Debug, Serialize, Deserialize)]
pub struct TxOut {
    value: Amount,
    scriptpubkey: Vec<u8>,
    scriptpubkey_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct TxIn {
    /// The transaction ID of the output that created this tx
    txid: TxId,
    // index of the referenced output
    vout: u32,
    /// This is needed to unlock the txout for spending
    scriptsig: Vec<u8>,
    /// A value to help miners decide which transactions are preferable to spend
    sequence: u32,
}

/// Mining a Bitcoin transaction is done
/// Fill a candidate block with transactions
/// Transactions can be gotten from the mempool
/// 

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    // header: Header,
    transactions: Vec<Transaction>
}

fn main() {
    // Need to be able to read the files from the mempool folder they're located in
    let manifest_path = PathBuf::from("mempool");

    // Pick transaction in the mempool, based on index.
    // Delete processed transaction(s) so other transactions will be processed.
    match fs::read_dir(manifest_path) {
        Ok(entries) => {
            for (index, entry) in entries.enumerate() {
                if index >= 20 {
                    break;
                }
                let entry = entry.unwrap();
                let path = entry.path();
                let file_name = &path.file_name().unwrap().to_str().unwrap();
                println!("File name: {}", file_name);

                // read the file to process the transaction
                let transaction_content = fs::read_to_string(path).unwrap();

                // expected content is a JSON string.
                // Parse JSON using serde crate
                let transaction: serde_json::error::Result<Transaction> = serde_json::from_str(transaction_content.as_str());
                match transaction {
                    Ok(tx) => {
                        println!("Transaction: {:?}", tx);
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
