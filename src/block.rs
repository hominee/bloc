use crate::blockchain::*;
use crate::constant::*;
use crate::transaction::Transaction;
use bitcoin_hashes::{sha256, sha256::Hash as Sha256, Hash, HashEngine};
use chrono::prelude::*;
use std::str::FromStr;

/// represent a Block that pushed to BlockChain
//#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub timestamp: DateTime<Utc>,
    pub data: Vec<Transaction>,
    pub prev_hash: Sha256,
    pub hash: Sha256,
    pub nonce: u64,
}

impl Block {
    pub fn new(timestamp: DateTime<Utc>, data: Vec<Transaction>) -> Self {
        let mut item = Self {
            timestamp,
            data,
            prev_hash: Sha256::from_slice(&[0u8; 32]).unwrap(),
            hash: Sha256::from_slice(&[0u8; 32]).unwrap(),
            nonce: 0,
        };
        item.hash = item.get_hash();
        item
    }

    pub fn get_hash(&self) -> Sha256 {
        let mut hash_engine = sha256::HashEngine::default();
        hash_engine.input(&self.prev_hash);
        hash_engine.input(
            &bincode::serialize(&self.timestamp).expect("failed to serialize block timestamp"),
        );
        hash_engine.input(
            &bincode::serialize(&self.data).expect("failed to serialize block transactions"),
        );
        hash_engine
            .input(&bincode::serialize(&self.nonce).expect("failed to serialize block nonce"));
        sha256::Hash::from_engine(hash_engine)
    }

    pub fn is_mined(&self, difficulty: u32) -> bool {
        assert!(difficulty < 32, "difficulty must less than 32");
        let hash = self.hash.as_ref();
        hash[0..difficulty as usize]
            .iter()
            .fold(true, |acc, ele| acc && ele == &u8::from_str(&"0").unwrap())
    }

    pub fn mine(&mut self, difficulty: u32) {
        while !self.is_mined(difficulty) {
            self.nonce += 1;
            let hash = self.get_hash();
            self.hash = hash;
        }
    }

    pub fn has_valid_transaction(&self, chain: &BlockChain) -> bool {
        let mut tips = 0.0;
        let mut reward = 0.0;
        self.data.iter().for_each(|trans| {
            if trans.from.eq(&MINT_PUBLIC_ADDRESS) {
                tips += trans.tips;
            } else {
                reward = trans.amount;
            }
        });

        reward - tips == chain.reward
            && self
                .data
                .iter()
                .fold(false, |acc, ele| acc && ele.is_valid(chain))
            && self
                .data
                .iter()
                .filter(|trans| trans.from.eq(&MINT_PUBLIC_ADDRESS))
                .collect::<Vec<_>>()
                .len()
                == 1
    }
}

#[cfg(test)]
mod test_block {
    use crate::block::*;
    use bitcoin_hashes::hex::ToHex;

    #[test]
    fn test_new() {
        let now = chrono::Utc::now();
        let trans = Vec::new();
        let item = Block::new(now, trans);
        assert_eq!(item.timestamp, now);
        assert_ne!(item.prev_hash, item.hash);
        assert_eq!(item.nonce, 0);
    }

    #[test]
    fn test_mine() {
        let now = chrono::Utc::now();
        let trans = Vec::new();
        let mut item = Block::new(now, trans);
        item.mine(1);
        assert_ne!(item.nonce, 0);
        assert!(item.hash.to_hex().starts_with("0"), "must start with 0");
    }
}
