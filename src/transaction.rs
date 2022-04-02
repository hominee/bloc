use crate::{blockchain::*, constant::*};
use bitcoin_hashes::{sha256, Hash as Sha256, HashEngine};
//use secp256k1::{ecdsa::Signature, KeyPair, Message, PubKey, *};
use crate::secp256k1::{KeyPair, PubKey, ToHex};
use k256::ecdsa::Signature;
use serde::{Deserialize, Serialize};

/// represent a transaction sent by a peer
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Transaction {
    pub from: PubKey,
    pub to: PubKey,
    pub amount: f64,
    pub tips: f64,
    pub signature: Signature,
}

impl Transaction {
    pub fn new(from: PubKey, to: PubKey, amount: f64, tips: Option<f64>) -> Self {
        let tips = tips.unwrap_or(0.0);
        Self {
            from,
            to,
            amount,
            tips,
            signature: MINT_KEY.3,
        }
    }

    pub fn transaction_msg(&self) -> Result<sha256::Hash, Box<dyn std::error::Error>> {
        let mut hash_engine = sha256::HashEngine::default();
        hash_engine.input(&self.from.as_ref());
        hash_engine.input(&self.to.as_ref());
        hash_engine.input(
            &bincode::serialize(&self.amount).expect("failed to serialize transaction amount"),
        );
        hash_engine
            .input(&bincode::serialize(&self.tips).expect("failed to serialize transaction tips"));
        Ok(sha256::Hash::from_engine(hash_engine))
    }

    pub fn is_valid(&self, chain: &BlockChain) -> bool {
        let msg = self.transaction_msg();
        if msg.is_err() {
            log::error!("digest the msg failed");
            return false;
        }
        let msg = msg.unwrap().as_inner().clone();
        /*
         *dbg!(
         *    self.from.serialize()[..32] != [0u8; 32],
         *    self.to.serialize()[..32] != [0u8; 32],
         *    chain.get_balance(&self.from) >= &self.amount + &self.tips
         *        || self.from.eq(&MINT_PUBLIC_ADDRESS),
         *    SECP.verify_ecdsa(&msg.unwrap(), &self.signature, &self.from)
         *        .is_ok()
         *);
         */
        log::debug!(
            "balance check: {}",
            (chain.get_balance(&self.from) >= &self.amount + &self.tips
                || self.from.eq(&MINT_PUBLIC_ADDRESS))
        );
        log::debug!(
            "verify check: {}",
            self.from.verify(&msg.as_slice(), &self.signature).is_ok()
        );
        !self.from.to_hex().starts_with(&ZEROKEY as &str)
            && self.to.to_hex() != &ZEROKEY as &str
            && self.amount != 0.0
            && (chain.get_balance(&self.from) >= &self.amount + &self.tips
                || self.from.eq(&MINT_PUBLIC_ADDRESS))
            && self.from.verify(&msg.as_slice(), &self.signature).is_ok()
    }

    pub fn sign(&mut self, key_pair: &KeyPair) {
        let public_key = key_pair
            .to_public_key()
            .expect("failed to obtain public key from key pair");
        if public_key == self.from {
            let msg = self.transaction_msg().expect("failed to hash Transaction");
            log::debug!("sign msg: {}", msg);
            let secret_key = key_pair
                .to_secret_key()
                .expect("failed to obtain secret key from key pair");
            self.signature = secret_key.sign(&msg);
            assert!(
                public_key.verify(&msg, &self.signature).is_ok(),
                "not verify"
            );
        } else {
            log::debug!("public_key != from with");
            dbg!(
                "the public key not matched: {} | {}",
                public_key.to_hex(),
                self.from.to_hex()
            );
        }
    }
}

#[cfg(test)]
mod test_transaction {
    use crate::secp256k1::{ToHex, *};
    use crate::transaction::*;

    #[test]
    fn test_new() {
        let mut secp = Secp256K1::new();
        let (_, pub1) = secp.gen_keypair();
        let (_, pub2) = secp.gen_keypair();
        let from = pub1;
        let to = pub2;
        let amount = 3000.0;
        let item = Transaction::new(from, to, amount, None);
        assert_eq!(item.from, pub1);
        assert_eq!(item.to, pub2);
        assert_eq!(item.amount, amount);
        assert_eq!(item.tips, 0.0);
        assert_eq!(item.signature.to_bytes(), MINT_KEY.3.to_bytes());
    }

    #[test]
    fn test_transaction_msg() {
        let mut secp = Secp256K1::new();
        let (_, pub1) = secp.gen_keypair();
        let (_, pub2) = secp.gen_keypair();
        let from = pub1;
        let to = pub2;
        let amount = 3000.0;
        let item = Transaction::new(from, to, amount, None);
        let msg = item.transaction_msg();
        assert!(msg.is_ok(), "failed to hash transaction into message");
        let msg = msg.unwrap();
        dbg!(&msg);
        assert_eq!(msg.to_vec().len(), 32);
    }

    #[test]
    fn test_sign() {
        let mut secp = Secp256K1::new();
        let (secret1, pub1) = secp.gen_keypair();
        let key_pair1 = KeyPair::from(&secret1);
        assert_eq!(
            key_pair1.to_public_key().unwrap(),
            pub1,
            "it should equal for the public key"
        );
        let (_, pub2) = secp.gen_keypair();
        let amount = 3000.0;
        let mut item = Transaction::new(pub1, pub2, amount, None);
        item.sign(&key_pair1);
        assert_ne!(item.signature, MINT_KEY.3, "signature not signed");
    }
}
