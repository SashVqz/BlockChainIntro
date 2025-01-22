use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use crate::network::peer::Peer;
use crate::network::message::{NetworkMessage, BlockData, TransactionData};
use crate::utils::{hashing, encoding};

pub struct Node {
    pub address: SocketAddr,
    pub peers: HashMap<SocketAddr, Peer>,
    pub incomingMessages: mpsc::Receiver<NetworkMessage>,
    pub outgoingMessages: mpsc::Sender<NetworkMessage>,
    pub blockSizeLimit: usize,
    pub transactionPool: Vec<TransactionData>,
    pub blockchain: Vec<BlockData>,
}

impl Node {
    pub fn new(
        address: SocketAddr,
        receiver: mpsc::Receiver<NetworkMessage>,
        sender: mpsc::Sender<NetworkMessage>,
        blockSizeLimit: usize,
    ) -> Self {
        Self {
            address,
            peers: HashMap::new(),
            incomingMessages: receiver,
            outgoingMessages: sender,
            blockSizeLimit,
            transactionPool: Vec::new(),
            blockchain: Vec::new(),
        }
    }

    pub fn addPeer(&mut self, peer: Peer) {
        self.peers.insert(peer.address, peer);
    }

    pub async fn handleIncoming(&mut self) {
        while let Some(message) = self.incomingMessages.recv().await {
            match message {
                NetworkMessage::Transaction(tx) => self.processTransaction(tx).await,
                NetworkMessage::Block(block) => self.processBlock(block).await,
                _ => {}
            }
        }
    }

    async fn processTransaction(&mut self, tx: TransactionData) {
        if self.validateTransaction(&tx) {
            self.transactionPool.push(tx);
        }
        if self.calculatePoolSize() > self.blockSizeLimit {
            self.createAndBroadcastBlock().await;
        }
    }

    fn calculatePoolSize(&self) -> usize {
        self.transactionPool
            .iter()
            .map(|tx| encoding::serialize(tx).unwrap().len())
            .sum()
    }

    fn validateTransaction(&self, tx: &TransactionData) -> bool {
        !tx.sender.is_empty() && !tx.receiver.is_empty() && tx.amount > 0
    }

    async fn processBlock(&mut self, block: BlockData) {
        if self.validateBlock(&block) && block.index == self.blockchain.len() as u64 {
            self.blockchain.push(block);
        }
    }

    fn validateBlock(&self, block: &BlockData) -> bool {
        if block.index > 0 {
            let previousBlock = &self.blockchain[(block.index - 1) as usize];
            if block.previousHash != self.hashBlock(previousBlock) {
                return false;
            }
        }
        if block.merkleRoot != self.calculateMerkleRoot(&block.transactions) {
            return false;
        }
        true
    }

    pub async fn broadcast(&self, message: &NetworkMessage) {
        let serialized_msg = match encoding::serialize(message) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error serializing the message: {}", e);
                return;
            }
        };

        for peer in self.peers.values() {
            let peer_addr = peer.address;
            match tokio::net::TcpStream::connect(peer_addr).await {
                Ok(mut stream) => {
                    if let Err(e) = stream.write_all(&serialized_msg).await {
                        eprintln!("Error sending message to {}: {}", peer_addr, e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to {}: {}", peer_addr, e);
                }
            }
        }
    }

    async fn createAndBroadcastBlock(&mut self) {
        let previousHash = if let Some(lastBlock) = self.blockchain.last() {
            self.hashBlock(lastBlock)
        } else {
            String::new()
        };
        let block = BlockData {
            index: self.blockchain.len() as u64,
            previousHash,
            timestamp: Utc::now().timestamp(),
            merkleRoot: self.calculateMerkleRoot(&self.transactionPool),
            nonce: 0,
            transactions: self.transactionPool.clone(),
        };
        self.transactionPool.clear();
        self.blockchain.push(block.clone());
        self.broadcast(&NetworkMessage::Block(block)).await;
    }

    fn calculateMerkleRoot(&self, transactions: &[TransactionData]) -> String {
        if transactions.is_empty() {
            return String::new();
        }
        let mut hashes: Vec<String> = transactions
            .iter()
            .map(|tx| hashing::sha256(&encoding::serialize(tx).unwrap()))
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

    fn hashBlock(&self, block: &BlockData) -> String {
        let blockBytes = encoding::serialize(block).unwrap();
        hashing::sha256(&blockBytes)
    }
}