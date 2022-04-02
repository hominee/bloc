use k256::ecdsa::signature::Signer;
use k256::elliptic_curve::serde::{
    de::Deserializer,
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};
use k256::*;
use rand_core::OsRng;

pub struct Secp256K1 {
    rng: OsRng,
}
impl Secp256K1 {
    pub fn new() -> Self {
        Self { rng: OsRng }
    }

    pub fn gen_keypair(&mut self) -> (SecKey, PubKey) {
        let sec_key_raw = k256::ecdsa::SigningKey::random(&mut self.rng);
        let sec_key_raw_byte = &sec_key_raw.to_bytes();
        unsafe {
            let sec_key_raw_arr =
                std::mem::transmute::<k256::FieldBytes, [u8; 32]>(sec_key_raw_byte.clone());
            let sec_key = SecKey::new(&sec_key_raw_arr);
            let pub_key_raw = sec_key_raw.verifying_key();
            let pub_key_bytes = pub_key_raw.to_bytes();
            let pub_key_raw_arr =
                std::mem::transmute::<k256::CompressedPoint, [u8; 33]>(pub_key_bytes.clone());
            let pub_key = PubKey::new(&pub_key_raw_arr);
            (sec_key, pub_key)
        }
    }
}

pub trait ToHex {
    fn to_hex(&self) -> String;
    fn to_bytes(&self) -> &[u8];
}

#[derive(Serialize, PartialEq, Debug, Clone, Copy)]
pub struct SecKey([u8; 32]);
impl SecKey {
    pub fn new(key: &[u8; 32]) -> Self {
        Self(*key)
    }

    pub fn from_slice(slice: &[u8; 32]) -> Self {
        Self(*slice)
    }

    pub fn from_hex(hex: &str) -> Result<Self, Error> {
        if hex.len() != 64 {
            return Err(Error {
                desc: "must be 64-bit long hex sequence".to_owned(),
            });
        }
        let mut bytes = [0u8; 32];
        (0..hex.len()).step_by(2).for_each(|i| {
            let digit = u8::from_str_radix(&hex[i..i + 2], 16).unwrap();
            let ind = i / 2;
            bytes[ind] = digit;
        });
        Ok(SecKey::from_slice(&bytes))
    }

    pub fn sign(&self, msg: &[u8]) -> k256::ecdsa::Signature {
        let sign_key = k256::ecdsa::SigningKey::from_bytes(&self.0).unwrap();
        sign_key.sign(msg)
    }
}
impl ToHex for SecKey {
    fn to_hex(&self) -> String {
        use std::fmt::Write;
        let mut ret = String::with_capacity(2 * self.0.len());
        self.0
            .iter()
            .for_each(|ch| write!(ret, "{:02x}", ch).expect("writing to string"));
        ret
    }

    fn to_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct PubKey(#[serde(with = "serde_pubkey")] [u8; 33]);
impl PubKey {
    pub fn new(key: &[u8; 33]) -> Self {
        Self(*key)
    }

    pub fn from_slice(slice: &[u8; 33]) -> Self {
        Self(*slice)
    }

    pub fn from_hex(hex: &str) -> Result<Self, Error> {
        //assert_eq!(hex.len(), 66, "must be 64-bit long hex sequence");
        if hex.len() != 66 {
            return Err(Error {
                desc: "must be 66-bit long hex sequence".to_owned(),
            });
        }
        let mut bytes = [0u8; 33];
        (0..hex.len()).step_by(2).for_each(|i| {
            let digit = u8::from_str_radix(&hex[i..i + 2], 16).unwrap();
            let ind = i / 2;
            bytes[ind] = digit;
        });
        Ok(PubKey::from_slice(&bytes))
    }

    pub fn verify(&self, msg: &[u8], signature: &ecdsa::Signature) -> Result<(), ecdsa::Error> {
        let verkey = k256::ecdsa::VerifyingKey::from_sec1_bytes(&self.0)?;
        use k256::ecdsa::signature::Verifier;
        unsafe {
            let bytes = verkey.to_bytes();
            let arr = std::mem::transmute::<k256::CompressedPoint, [u8; 33]>(bytes);
            assert_eq!(&arr, &self.0);
        }
        verkey.verify(msg, signature)
    }

    pub fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl ToHex for PubKey {
    fn to_hex(&self) -> String {
        use std::fmt::Write;
        let mut ret = String::with_capacity(2 * self.0.len());
        self.0
            .iter()
            .for_each(|ch| write!(ret, "{:02x}", ch).expect("writing to string"));
        ret
    }

    fn to_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

mod serde_pubkey {
    use super::*;
    pub fn serialize<S>(key: &[u8; 33], s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(33))?;
        for e in key {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(d: D) -> Result<[u8; 33], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes_raw = <&[u8]>::deserialize(d).expect("failed to deserialize to bytes");
        assert_eq!(bytes_raw.len(), 33, "must be 33 u8");
        let mut bytes = [0; 33];
        for ind in 0..33 {
            bytes[ind] = bytes_raw[ind];
        }
        Ok(bytes)
    }
}

//#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[derive(PartialEq, Debug, Clone)]
pub struct KeyPair(elliptic_curve::JwkEcKey);
impl From<&SecKey> for KeyPair {
    fn from(sec_key: &SecKey) -> Self {
        let sign_key = k256::ecdsa::SigningKey::from_bytes(&sec_key.0).expect("invalid secret key");
        let key: elliptic_curve::SecretKey<k256::Secp256k1> =
            elliptic_curve::SecretKey::from(&sign_key);
        Self(elliptic_curve::JwkEcKey::from(key))
    }
}
impl KeyPair {
    /// Is this JWK a keypair that includes a private key?
    pub fn is_keypair(&self) -> bool {
        self.0.is_keypair()
    }

    pub fn to_public_key(&self) -> Result<PubKey, Box<dyn std::error::Error>> {
        let raw_key = self.0.to_public_key()?;
        let verkey = k256::ecdsa::VerifyingKey::from(raw_key);
        unsafe {
            let raw_bytes =
                std::mem::transmute::<k256::CompressedPoint, [u8; 33]>(verkey.to_bytes());
            Ok(PubKey::from_slice(&raw_bytes))
        }
    }

    pub fn to_secret_key(&self) -> Result<SecKey, Box<dyn std::error::Error>> {
        let seckey: elliptic_curve::SecretKey<k256::Secp256k1> = self.0.to_secret_key()?;
        unsafe {
            let bytes = std::mem::transmute::<k256::FieldBytes, [u8; 32]>(seckey.to_be_bytes());
            Ok(SecKey::new(&bytes))
        }
    }
}

impl ToHex for ecdsa::Signature {
    fn to_hex(&self) -> String {
        unsafe {
            let bytes = std::mem::transmute::<ecdsa::Signature, [u8; 64]>(*self);
            use std::fmt::Write;
            let mut ret = String::with_capacity(128);
            bytes
                .iter()
                .for_each(|ch| write!(ret, "{:02x}", ch).expect("writing to string"));
            ret
        }
    }

    fn to_bytes(&self) -> &[u8] {
        unsafe { std::mem::transmute::<&ecdsa::Signature, &[u8; 64]>(self) }
    }
}

pub struct Error {
    pub desc: String,
}
use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {{ {} }}", self.desc)
    }
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.desc, f)
    }
}
impl std::error::Error for Error {}

#[cfg(test)]
mod test_all {
    use super::*;

    #[test]
    fn test_int() {
        let mut secp = Secp256K1::new();
        let (seckey, pubkey) = secp.gen_keypair();
        dbg!(seckey.to_hex(), pubkey.to_hex());
        let keypair = KeyPair::from(&seckey);
        assert_eq!(seckey.0, keypair.to_secret_key().unwrap().0);
        assert_eq!(pubkey.0, keypair.to_public_key().unwrap().0);
        let msg = b"hello world bloc";
        let signature = seckey.sign(msg);
        let ver_res = pubkey.verify(msg, &signature);
        assert!(ver_res.is_ok());
    }
}
