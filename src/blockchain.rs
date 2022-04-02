use crate::secp256k1::*;
use crate::{block::Block, constant::*, transaction::Transaction};
use std::sync::Once;

#[derive(PartialEq)]
pub struct BlockChain {
    pub transactions: Vec<Transaction>,
    pub chain: Vec<Block>,
    pub difficulty: u32,
    pub block_time: u32,
    pub reward: f64,
}

impl Clone for BlockChain {
    fn clone(&self) -> Self {
        Self {
            transactions: self.transactions.clone(),
            chain: self.chain.clone(),
            difficulty: 1,
            block_time: self.block_time.clone(),
            reward: self.reward.clone(),
        }
    }
}

impl BlockChain {
    #[allow(dead_code)]
    pub fn new() -> Self {
        static mut INITIALCOINRELEASE: Option<Transaction> = None;
        static INIT: Once = Once::new();
        unsafe {
            INIT.call_once(|| {
                INITIALCOINRELEASE.replace(Transaction {
                    from: *MINT_PUBLIC_ADDRESS,
                    to: KEY1.1,
                    amount: 100000.0,
                    tips: 0.0,
                    signature: MINT_KEY.3,
                });
            })
        };

        let now = chrono::Utc::now();
        let block = unsafe { INITIALCOINRELEASE.clone().unwrap() };
        let block = Block::new(now, vec![block]);
        Self {
            transactions: Vec::new(),
            chain: vec![block],
            //chain: vec![],
            difficulty: 1,
            block_time: 30000,
            reward: *REWARD,
        }
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn get_last_block(&self) -> &Block {
        self.chain.last().expect("failed to obtain the last block")
    }

    pub fn add_block(&mut self, mut block: Block) {
        block.prev_hash = self.get_last_block().hash.clone();
        block.hash = block.get_hash();
        block.mine(self.difficulty);
        log::debug!("mined with hash: {}", block.hash);
        self.chain.push(block);
        if chrono::Utc::now() > self.get_last_block().timestamp {
            self.difficulty += 1;
        } else {
            self.difficulty -= 1;
        };
    }

    pub fn add_transaction(&mut self, trans: Transaction) {
        if trans.is_valid(self) {
            self.transactions.push(trans);
        } else {
            log::error!("failed to add transaction to blockchain");
            dbg!("failed to add transaction to blockchain");
        }
    }

    pub fn mine_transaction(&mut self, reward_address: &PubKey) -> Block {
        let mut tips = 0.0;
        self.transactions.iter().for_each(|trans| {
            tips += trans.tips;
        });

        let mut reward = Transaction::new(
            MINT_PUBLIC_ADDRESS.clone(),
            reward_address.clone(),
            self.reward + tips,
            //Some(tips),
            None,
        );
        reward.sign(&MINT_KEY_PAIR);
        let mut block_transactions = Vec::new();
        if !self.transactions.is_empty() {
            std::mem::swap(&mut block_transactions, &mut self.transactions);
            assert!(self.transactions.is_empty(), "must be empty after swap");
        }
        block_transactions.push(reward);
        let block = Block::new(chrono::Utc::now(), block_transactions);
        self.add_block(block.clone());
        block
    }

    pub fn get_balance(&self, address: &PubKey) -> f64 {
        let mut balance = 0.0;
        self.chain.iter().for_each(|block| {
            block.data.iter().for_each(|trans| {
                if &trans.from == address {
                    balance -= trans.amount;
                    balance -= trans.tips;
                }
                if &trans.to == address {
                    balance += trans.amount;
                }
            });
        });
        dbg!(&balance);
        balance
    }

    pub fn is_valid(&self, chain: &BlockChain) -> bool {
        for index in 1..chain.chain.len() {
            let current_block = &chain.chain[index];
            let prev_block = &chain.chain[index - 1];
            let hash = current_block.get_hash();
            if &current_block.hash != &hash
                || prev_block.hash.ne(&current_block.prev_hash)
                || !current_block.has_valid_transaction(chain)
            {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod test_block_chain {
    use crate::blockchain::*;
    use crate::constant::*;
    use crate::secp256k1::*;

    // make sure the INITIALCOINRELEASE is static
    #[test]
    fn test_new() {
        let item = BlockChain::new();
        assert_eq!(item.chain.len(), 1);
        let item_du = BlockChain::new();
        assert_eq!(item.chain[0].data, item_du.chain[0].data);
        assert_eq!(item.transactions, item_du.transactions);
        assert_eq!(item.difficulty, item_du.difficulty);
        assert_eq!(item.block_time, item_du.block_time);
        assert_eq!(item.reward, item_du.reward);
    }

    #[test]
    fn test_add_block() {
        let mut item = BlockChain::new();
        let mut secp = Secp256K1::new();
        let (_, pub1) = secp.gen_keypair();
        let (_, pub2) = secp.gen_keypair();
        let transaction = Transaction {
            from: pub1,
            to: pub2,
            amount: 3000.0,
            tips: 100.0,
            signature: MINT_KEY.3,
        };
        let now = chrono::Utc::now();
        let block = Block::new(now, vec![transaction.clone()]);
        item.add_block(block);
        //dbg!(&item.chain);
        assert_eq!(item.chain[1].data[0], transaction);
    }

    #[test]
    fn test_mine_trasaction() {
        let mut chain = BlockChain::new();
        let mut secp = Secp256K1::new();
        let (_, pub1) = secp.gen_keypair();
        chain.mine_transaction(&pub1);
        dbg!(&chain.chain[1].data[0]);
        assert_eq!(chain.chain[1].data[0].from, *MINT_PUBLIC_ADDRESS);
        assert_eq!(chain.chain[1].data[0].to, pub1);
    }

    #[test]
    fn test_get_balance() {
        let mut chain = BlockChain::new();
        let mut transaction = Transaction {
            from: KEY1.1,
            to: KEY2.1,
            amount: 333.0,
            tips: 10.0,
            signature: MINT_KEY.3,
        };
        transaction.sign(&KEY1.2);
        dbg!(&transaction,);
        chain.add_transaction(transaction);
        chain.mine_transaction(&KEY2.1);
        let balance1 = chain.get_balance(&KEY1.1);
        let balance2 = chain.get_balance(&KEY2.1);
        dbg!(&chain.chain);
        assert_eq!(balance1, 99657.0);
        assert_eq!(balance2, 640.0);
    }
}
