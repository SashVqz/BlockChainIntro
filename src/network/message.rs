use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkMessage {
    Transaction(TransactionData),
    Block(BlockData),
    StakeUpdate(StakeUpdateData),
    ValidatorRequest,
    ValidatorResponse(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionData {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockData {
    pub index: u64,
    pub previousHash: String,
    pub timestamp: i64,
    pub merkleRoot: String,
    pub nonce: u64,
    pub transactions: Vec<TransactionData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StakeUpdateData {
    pub validator: String,
    pub amount: u64,
}