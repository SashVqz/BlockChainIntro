use serde::{Serialize, Deserialize};
use rand::Rng;
use crate::utils::{hashing, encoding};

#[derive(Debug, Clone)]
pub struct Polynomial {
    pub coefficients: Vec<i64>,
    pub modulus: i64,
}

impl Polynomial {
    pub fn new(coefficients: Vec<i64>, modulus: i64) -> Self {
        Polynomial {
            coefficients: coefficients.into_iter().map(|c| c % modulus).collect(),
            modulus,
        }
    }

    pub fn add(&self, other: &Polynomial) -> Self {
        let coefficients = self
            .coefficients
            .iter()
            .zip(&other.coefficients)
            .map(|(a, b)| (a + b) % self.modulus)
            .collect();
        Polynomial::new(coefficients, self.modulus)
    }

    pub fn multiply(&self, other: &Polynomial) -> Self {
        let n = self.coefficients.len();
        let mut result = vec![0; 2 * n - 1];

        for i in 0..n {
            for j in 0..n {
                result[i + j] = (result[i + j] + self.coefficients[i] * other.coefficients[j]) % self.modulus;
            }
        }

        Polynomial::new(result[..n].to_vec(), self.modulus)
    }
}

pub struct KeyPair {
    pub publicKey: Vec<Polynomial>,
    pub secretKey: Vec<Polynomial>,
}

impl KeyPair {
    pub fn generate(modulus: i64, size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let publicPolynomials = (0..size)
            .map(|_| Polynomial::new((0..size).map(|_| rng.gen_range(0..modulus)).collect(), modulus))
            .collect::<Vec<_>>();
        let secretPolynomials = (0..size)
            .map(|_| Polynomial::new((0..size).map(|_| rng.gen_range(0..modulus)).collect(), modulus))
            .collect::<Vec<_>>();
        let errors = (0..size)
            .map(|_| Polynomial::new((0..size).map(|_| rng.gen_range(0..modulus)).collect(), modulus))
            .collect::<Vec<_>>();

        let publicKey = publicPolynomials
            .iter()
            .zip(&secretPolynomials)
            .map(|(publicPoly, secretPoly)| publicPoly.multiply(secretPoly).add(&errors[0]))
            .collect::<Vec<_>>();

        KeyPair {
            publicKey,
            secretKey: secretPolynomials,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        Transaction {
            sender,
            receiver,
            amount,
            signature: Vec::new(),
        }
    }

    pub fn sign(&mut self, keyPair: &KeyPair, modulus: i64, size: usize) {
        let dataToSign = self.getDataToSign();
        let hash = hashing::sha256(dataToSign.as_bytes());

        let signaturePolynomials = keyPair
            .secretKey
            .iter()
            .map(|secretPoly| {
                secretPoly.multiply(&Polynomial::new(
                    hash.as_bytes().iter().map(|&x| x as i64).collect(),
                    modulus,
                ))
            })
            .collect::<Vec<_>>();

        self.signature = encoding::serialize(&signaturePolynomials).expect("Failed to serialize the signature");
    }

    pub fn verify(&self, keyPair: &KeyPair, modulus: i64) -> bool {
        let dataToSign = self.getDataToSign();
        let hash = hashing::sha256(dataToSign.as_bytes());

        let signaturePolynomials: Vec<Polynomial> =
            encoding::deserialize(&self.signature).expect("Failed to deserialize the signature");
        let verificationCheck = keyPair
            .publicKey
            .iter()
            .zip(&signaturePolynomials)
            .map(|(publicPoly, signaturePoly)| publicPoly.multiply(signaturePoly))
            .collect::<Vec<_>>();

        verificationCheck.iter().all(|verificationPoly| {
            verificationPoly.coefficients
                == hash.as_bytes().iter().map(|&x| x as i64).collect::<Vec<_>>()
        })
    }

    fn getDataToSign(&self) -> String {
        format!("{}:{}:{}", self.sender, self.receiver, self.amount)
    }

    pub fn calculateHash(&self) -> String {
        let transactionBytes = encoding::serialize(self).expect("Failed to serialize transaction");
        hashing::sha256(&transactionBytes)
    }
}