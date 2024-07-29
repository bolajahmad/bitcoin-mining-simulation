use std::path::PathBuf;
use std::fs;

use bitcoin::{
    block::{Header, Version as HVersion}, transaction::Version, Amount, BlockHash, CompactTarget, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxMerkleNode, TxOut, Txid, Witness
};
use sha2::{Sha256, Digest};
use bitcoin_hashes::{sha256, sha256d, Hash};
// use bitcoin::{block::Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct RawInputs {
    // Transaction ID that created the UTXO
    txid: String,
    // Index of the UTXO
    vout: u32,
    // A script to decide the unlick condition of the UTXO
    scriptsig: String,
    sequence: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawOutputs {
    value: u32,
    scriptpubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawTransaction {
    /// The protocol version, is currently expected to be 1 or 2 (BIP 68).
    pub version: Version,
    /// List of transaction inputs
    pub vin: Vec<RawInputs>,
    /// List of transaction outputs
    pub vout: Vec<RawOutputs>,
}

impl RawTransaction {
    fn process_transaction(&self) -> Option<Transaction> {
        let mut tx = Transaction {
            version: self.version,
            lock_time: bitcoin::absolute::LockTime::from_height(0u32).unwrap(),
            input: Vec::new(),
            output: Vec::new(),
        };

        for input in self.vin.iter() {
            let txid = sha256d::Hash::from_byte_array(
                string_to_array_size32(input.txid.as_str())
            );
            let script_sig = ScriptBuf::from_hex(
                &input.scriptsig.to_owned()
                )
                .unwrap();

            let tx_in = TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_raw_hash(txid),
                    vout: input.vout,
                },
                script_sig,
                sequence: Sequence(input.sequence),
                witness: Witness::new()
            };

            tx.input.push(tx_in);
        }

        for output in self.vout.iter() {
            let script_pubkey = ScriptBuf::from_hex(
                &output.scriptpubkey.to_owned()
                )
                .unwrap();

            let tx_out = TxOut {
                value: Amount::from_int_btc(output.value.into()),
                script_pubkey,
            };

            tx.output.push(tx_out);
        }

        Some(tx)
    }
}

fn main() {
    // Need to be able to read the files from the mempool folder they're located in
    let manifest_path = PathBuf::from("mempool");

    let mut transactions: Vec<Transaction> = Vec::with_capacity(10);
    let mut tx_hashes: Vec<&str> = Vec::with_capacity(10);

    // Pick transaction in the mempool, based on index.
    // Delete processed transaction(s) so other transactions will be processed.
    match fs::read_dir(manifest_path) {
        Ok(entries) => {
            for (index, entry) in entries.enumerate() {
                if index >= 10 {
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
                // consider serde_json::from_reader
                let transaction: Result<RawTransaction, serde_json::Error> = serde_json::from_str(transaction_content.as_str());
                match transaction {
                    Ok(tx) => {
                        let processed_tx = tx.process_transaction();
                        match processed_tx {
                            Some(tx) => {
                                // let ntxid = tx.compute_ntxid().as_byte_array();
                                // tx_hashes.push(std::str::from_utf8(ntxid).unwrap());
                                transactions.push(tx);
                                // println!("Transaction: {:?}", ntxid);
                            }
                            None => {
                                println!("Error: Unable to process transaction");
                            }
                        }
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
    // Create the block header
    // Previous block hash
    let prev_blockhash = BlockHash::from_raw_hash(sha256d::Hash::from_byte_array(
        string_to_array_size32("0000000000000000000000000000000000000000000000000000000000000000")
    ));
    let merkle_root = calculate_merkle_root(tx_hashes);
    
    println!("Merkle Root: {:?}", merkle_root);
    let mut header = Header {
        version: HVersion::ONE,
        prev_blockhash,
        merkle_root: TxMerkleNode::from_raw_hash(sha256d::Hash::from_byte_array(string_to_array_size32(std::str::from_utf8(&merkle_root).unwrap()))),
        time: 0,
        bits: CompactTarget::from_hex("0").unwrap(),
        nonce: 0,
    };
}

fn pair_and_hash(hash1: &[u8], hash2: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(hash1);
    hasher.update(hash2);
    hasher.finalize().to_vec()
}

fn calculate_merkle_root(txs: Vec<&str>) -> Vec<u8> {
    let mut inner_txs = txs.clone();
    let mut new_txs: Vec<String> = Vec::new();
    while inner_txs.len() > 1 {
        if inner_txs.len() % 2 != 0 {
            inner_txs.push(txs.last().unwrap().clone());
        }

        
        // let mut temp_txs: Vec<String> = Vec::new();
        for i in (0..inner_txs.len()).step_by(2) {
            let hash1 = &inner_txs[i].as_bytes();
            let hash2 = &inner_txs[i + 1].as_bytes();

            println!("Generated Hashes: {:?}, {:?}", hash1, hash2);
            let combined_hash = pair_and_hash(
                hash1,
                hash2
            );
            let result = String::from_utf8(combined_hash).unwrap();
            new_txs.push(result);
        }
        
    }
    let txs = new_txs;

    txs[0].as_bytes().to_vec()
}

fn string_to_array_size32(input: &str) -> [u8; 32] {
    let bytes = input.as_bytes();
    let mut array = [0u8; 32];
    let len = if bytes.len() > 32 { 32 } else { bytes.len() };
    array[..len].copy_from_slice(&bytes[..len]);
    array
}