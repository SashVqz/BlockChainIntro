use serde::{Serialize, Deserialize};
use super::transaction::Transaction;
use chrono::prelude::*;
use crate::utils::{hashing, encoding};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub blockHeader: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub blockSize: Option<usize>, // Size of the block in bytes.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub timestamp: i64,
    pub prevHash: String,
    pub nonce: u64,
    pub merkleRoot: String,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, prevHash: String) -> Self {
        let blockHeader = BlockHeader {
            timestamp: Utc::now().timestamp(),
            prevHash,
            nonce: 0,
            merkleRoot: String::new(),
        };

        let mut block = Block { 
            blockHeader, 
            transactions,
            blockSize: None,
        };
        block.blockHeader.merkleRoot = block.calculateMerkleRoot();
        block.blockSize = Some(block.calculateBlockSize());
        block
    }

    pub fn calculateHash(&self) -> String {
        let headerBytes = encoding::serialize(&self.blockHeader)
            .expect("Failed to serialize the block header");
        hashing::sha256(&headerBytes)
    }

    fn calculateMerkleRoot(&self) -> String {
        if self.transactions.is_empty() {
            return String::new();
        }

        let mut hashes: Vec<String> = self.transactions
            .iter()
            .map(|tx| tx.calculateHash())
            .collect();

        while hashes.len() > 1 {
            let mut newHashes = Vec::new();
            for chunk in hashes.chunks(2) {
                let concatenated = if chunk.len() == 2 {
                    chunk[0].clone() + &chunk[1]
                } else {
                    chunk[0].clone() + &chunk[0]
                };
                newHashes.push(hashing::sha256(concatenated.as_bytes()));
            }
            hashes = newHashes;
        }

        hashes[0].clone()
    }

    pub fn mineBlock(&mut self, difficulty: usize) {
        let targetPrefix = "0".repeat(difficulty);
        while !self.calculateHash().starts_with(&targetPrefix) {
            self.blockHeader.nonce += 1;
        }
    }

    fn calculateBlockSize(&self) -> usize {
        encoding::serialize(&self)
            .map(|bytes| bytes.len())
            .unwrap_or(0)
    }
}