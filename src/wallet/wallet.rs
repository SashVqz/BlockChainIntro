use crate::wallet::keys::KeyPair;
use crate::utils::{hashing, encoding};
use crate::network::message::{BlockData, TransactionData};

pub struct Wallet {
    pub addresses: Vec<String>, 
    pub keypairs: Vec<KeyPair>, 
    pub balance: u64, 
}

impl Wallet {
    pub fn new() -> Self {
        let keypair = KeyPair::generate();
        let address = Wallet::generate_address(&keypair.public_key);
        Self {
            addresses: vec![address],
            keypairs: vec![keypair],
            balance: 0,
        }
    }

    pub fn generate_new_address(&mut self) -> String {
        let keypair = KeyPair::generate();
        let address = Wallet::generate_address(&keypair.public_key);
        self.addresses.push(address.clone());
        self.keypairs.push(keypair);
        address
    }

    fn generate_address(public_key: &[u8]) -> String {
        hashing::sha256(public_key)
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn sync_balance(&mut self, blockchain: &[BlockData]) {
        let mut new_balance = 0;

        for block in blockchain {
            for tx in &block.transactions {
                if self.addresses.contains(&tx.receiver) {
                    new_balance += tx.amount;
                }

                if self.addresses.contains(&tx.sender) {
                    new_balance = new_balance.saturating_sub(tx.amount);
                }
            }
        }

        self.balance = new_balance;
    }

    pub fn send_payment(
        &mut self,
        recipient: &str,
        amount: u64,
    ) -> Result<TransactionData, String> {
        if amount > self.balance {
            return Err("Insufficient funds".to_string());
        }

        let sender_index = 0;
        let sender_address = self
            .addresses
            .get(sender_index)
            .ok_or_else(|| "No address available".to_string())?;
        self.balance -= amount;

        let transaction_data = format!("{}:{}:{}", sender_address, recipient, amount);
        let signature = {
            let private_key = &self.keypairs[sender_index].private_key;
            let hash = hashing::sha256(transaction_data.as_bytes());
            encoding::serialize(&hash.iter().zip(private_key).map(|(h, k)| h ^ k).collect::<Vec<u8>>())
                .expect("Failed to serialize signature")
        };

        let tx = TransactionData {
            sender: sender_address.clone(),
            receiver: recipient.to_string(),
            amount,
            signature,
        };

        Ok(tx)
    }

    pub fn verify_transaction(transaction: &TransactionData, public_key: &[u8]) -> bool {
        let transaction_data = format!("{}:{}:{}", transaction.sender, transaction.receiver, transaction.amount);

        let hash = hashing::sha256(transaction_data.as_bytes());

        if let Ok(signature) = encoding::deserialize::<Vec<u8>>(&transaction.signature) {
            signature
                .iter()
                .zip(public_key)
                .all(|(s, p)| *s == (*p ^ hash[0]))
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_with_multiple_keys() {
        let mut wallet = Wallet::new();
        let new_address = wallet.generate_new_address();

        assert_eq!(wallet.addresses.len(), 2);
        assert_eq!(wallet.keypairs.len(), 2);
        assert!(!new_address.is_empty());
    }

    #[test]
    fn test_wallet_sync_with_full_blockchain() {
        let mut wallet = Wallet::new();
        wallet.balance = 0;

        let recipient = wallet.addresses[0].clone();

        let blockchain = vec![
            BlockData {
                index: 0,
                previousHash: String::new(),
                timestamp: 0,
                merkleRoot: String::new(),
                nonce: 0,
                transactions: vec![
                    TransactionData {
                        sender: "sender_1".to_string(),
                        receiver: recipient.clone(),
                        amount: 50,
                        signature: Vec::new(),
                    },
                ],
            },
            BlockData {
                index: 1,
                previousHash: String::new(),
                timestamp: 0,
                merkleRoot: String::new(),
                nonce: 0,
                transactions: vec![
                    TransactionData {
                        sender: "sender_2".to_string(),
                        receiver: recipient.clone(),
                        amount: 75,
                        signature: Vec::new(),
                    },
                    TransactionData {
                        sender: recipient.clone(),
                        receiver: "other_address".to_string(),
                        amount: 30,
                        signature: Vec::new(),
                    },
                ],
            },
        ];

        wallet.sync_balance(&blockchain);

        assert_eq!(wallet.get_balance(), 95);
    }

    #[test]
    fn test_transaction_signature() {
        let mut wallet = Wallet::new();
        wallet.balance = 100;

        let recipient = "recipient_address";
        let tx = wallet.send_payment(recipient, 50).unwrap();

        let public_key = &wallet.keypairs[0].public_key;
        assert!(Wallet::verify_transaction(&tx, public_key));
    }
}