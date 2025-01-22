use serde::{Serialize, Deserialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub address: SocketAddr,
    pub is_super_node: bool, 
    pub stake: u64,
}

impl Peer {
    pub fn new(address: SocketAddr, is_super_node: bool, stake: u64) -> Self {
        Self {
            address,
            is_super_node,
            stake,
        }
    }

    pub fn is_super_node(&self) -> bool {
        self.is_super_node
    }

    pub fn update_stake(&mut self, new_stake: u64) {
        self.stake = new_stake;
    }

    pub fn has_valid_stake(&self) -> bool {
        self.stake > 0
    }
}