use crate::secp256k1::{KeyPair, PubKey, SecKey, Secp256K1};
use k256::*;
use std::sync::Mutex;

lazy_static! {
    pub static ref REWARD: f64 = 100.0;
    pub static ref ZEROKEY: String = "00000000000000000000000000000000000000000000000000000000000000000000000000000000".to_owned();
    pub static ref SECP: Mutex<Secp256K1> = Mutex::new(Secp256K1::new());
    pub static ref MINT_KEY: (SecKey, PubKey, KeyPair, ecdsa::Signature) = {
        let (secret, public) = SECP.lock().unwrap().gen_keypair();
        let key_pair = KeyPair::from(&secret);
        let raw_bytes = [123u8; 64];
        let signature = secret.sign(&raw_bytes);
        //dbg!(&secret.as_ref().len(), &secret.len());
        //dbg!(&public.to_hex().len(), &public.serialize().len());
        (secret, public, key_pair, signature)
    };
    pub static ref MINT_PRIVATE_KEY: SecKey = MINT_KEY.0;
    pub static ref MINT_PUBLIC_ADDRESS: PubKey = MINT_KEY.1;
    pub static ref MINT_KEY_PAIR: KeyPair = MINT_KEY.2.clone();
    pub static ref KEY1: (SecKey, PubKey, KeyPair) = {
        let (secret, public) = SECP.lock().unwrap().gen_keypair();
        let key_pair = KeyPair::from(&secret);
        (secret, public, key_pair)
    };
    pub static ref KEY2: (SecKey, PubKey, KeyPair) = {
        let (secret, public) = SECP.lock().unwrap().gen_keypair();
        let key_pair = KeyPair::from(&secret);
        (secret, public, key_pair)
    };
}
