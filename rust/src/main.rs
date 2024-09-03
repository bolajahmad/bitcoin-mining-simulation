use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs::File, path::PathBuf};
use std::fs;

use base64::write;
use bitcoin::consensus::Encodable;
use bitcoin::hex::DisplayHex;
use bitcoin::Target;
use bitcoin::{
    block::{Header, Version as HVersion}, transaction::Version, 
    Amount, Block, BlockHash, CompactTarget, OutPoint, ScriptBuf,
    Sequence, Transaction, TxIn, TxMerkleNode, TxOut, Txid, Witness,
    hashes::sha256d::Hash as DHash,
};
use sha2::{Sha256, Digest};
use std::io::Write;
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

fn define_coinbase_tx() -> Transaction {
    let input: Vec<TxIn> = Vec::from([TxIn {
        previous_output: OutPoint::default(),
        script_sig: ScriptBuf::default(),
        sequence: Sequence::MAX,
        witness: Witness::from_slice(&vec![b"0000000", b"1233212"].to_vec())
    }]);

    let output: Vec<TxOut> = Vec::from([
        TxOut {
            value: Amount::from_int_btc(50),
            script_pubkey: ScriptBuf::new(),
        },
        TxOut {
            value: Amount::from_int_btc(50),
            script_pubkey: ScriptBuf::new(),
        }
    ]);

    Transaction {
        version: Version(1),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input,
        output,
    }
}

fn main() {
    // Need to be able to read the files from the mempool folder they're located in
    let manifest_path = PathBuf::from("mempool");

    let mut transactions: Vec<Transaction> = Vec::with_capacity(10);

    // push coinbase transaction
    let coinbase = define_coinbase_tx();
    transactions.push(coinbase);

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
                                transactions.push(tx);
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
    // println!("Blockhash value gotten: {:?}", string_to_array_size32());
    // Create the block header
    // Previous block hash
    let prev_blockhash = BlockHash::from_raw_hash(DHash::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap());
    let header = Header {
        version: HVersion::from_consensus(5),
        prev_blockhash,
        merkle_root: TxMerkleNode::from_byte_array(string_to_array_size32("0000000000000000000000000000000000000000000000000000000000000000")),
        time: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32,
        bits: Target::from_hex("0x0000ffff00000000000000000000000000000000000000000000000000000000").unwrap().to_compact_lossy(),
        nonce: 0,
    };
    let mut block = Block {
        header,
        txdata: transactions
    };
    block.header.merkle_root = block.compute_merkle_root().unwrap();

    println!("Block target: {:?}", header.target());

    let hash = block.block_hash();
    println!("Hash of the Block, {:?}", hash);
    
    println!("Mining the block");
    // Increment nonce till target is met
    while !block.header.target().is_met_by(block.block_hash()) {
        // println!("Nonce: {}", block.header.nonce);
        block.header.nonce += 1;
    }

    // Create out.txt file
    let mut file = File::create("out.txt").expect("Failed to create out.txt");

    println!("Created file and writing header into");
    // encode the block_header and write to out.txt
    let encoded_header = &mut Vec::new();
    block.header.consensus_encode(encoded_header).expect("Failed to encode header");
    writeln!(file, "{}", encoded_header.as_hex()).expect("Write header to file failed");

    // Write serialized coinbase transaction to out.txt
    let coinbasetx = block.coinbase().unwrap();
    let mut encoded_coinbase = Vec::new();
    coinbasetx.consensus_encode(&mut encoded_coinbase).expect("Failed to encode coinbase");
    writeln!(file, "{}", encoded_coinbase.as_hex()).expect("Write coinbase to file failed");

    // Write transaction IDs
    for tx in block.txdata.iter() {
        writeln!(file, "{}", tx.compute_txid()).expect("Write transaction to file failed");
    }
}

fn string_to_array_size32(input: &str) -> [u8; 32] {
    let bytes = input.as_bytes();
    let mut array = [0u8; 32];
    let len = if bytes.len() > 32 { 32 } else { bytes.len() };
    array[..len].copy_from_slice(&bytes[..len]);
    array
}