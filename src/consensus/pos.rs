use std::collections::HashMap;
use rand::Rng;
use crate::utils::hashing;

pub struct ProofOfStake {
    stakes: HashMap<String, u64>,  
    validatorHistory: Vec<String>, 
    maxHistorySize: usize, 
}

impl ProofOfStake {
    pub fn new(maxHistorySize: usize) -> Self {
        ProofOfStake {
            stakes: HashMap::new(),
            validatorHistory: Vec::new(),
            maxHistorySize,
        }
    }

    pub fn setStake(&mut self, validator: String, amount: u64) {
        self.stakes.insert(validator, amount);
    }

    pub fn selectValidator(&mut self, lastBlockHash: String) -> Option<String> {
        let totalStake: u64 = self.stakes.values().sum();
        if totalStake == 0 {
            return None;
        }

        let randomSeed = self.generateRandomSeed(&lastBlockHash);
        let mut threshold = randomSeed % totalStake;
        for (validator, stake) in &self.stakes {
            if threshold < *stake {
                if self.validatorHistory.contains(validator) {
                    continue;
                }
                self.addToHistory(validator.clone());
                return Some(validator.clone());
            }
            threshold -= stake;
        }
        None
    }

    pub fn slash(&mut self, validator: &String, penalty: u64) {
        if let Some(stake) = self.stakes.get_mut(validator) {
            *stake = stake.saturating_sub(penalty);
        }
    }

    fn addToHistory(&mut self, validator: String) {
        self.validatorHistory.push(validator);
        if self.validatorHistory.len() > self.maxHistorySize {
            self.validatorHistory.remove(0); 
        }
    }

    fn generateRandomSeed(&self, lastBlockHash: &str) -> u64 {
        let combined = format!("{}{}", lastBlockHash, rand::thread_rng().gen::<u64>());
        let hash = hashing::sha256(combined.as_bytes());
        u64::from_be_bytes(hash[0..8].try_into().unwrap())
    }
}