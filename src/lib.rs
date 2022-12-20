mod hash;
mod key;
mod time;
pub(crate) mod util;

pub use hash::Hash;
pub use key::{PrivateKey, PublicKey, Signature};
pub use time::UtcDateTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockData {
    CreateWallet {
        id: u64,
        key: PublicKey,
        time: UtcDateTime,
        signature: Signature,
    },
    Transaction {
        fee: u64,
        amount: u64,
        time: UtcDateTime,
        sender: PublicKey,
        receiver: PublicKey,
        signature: Signature,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: u64,
    pub nonce: u64,
    pub miner: PublicKey,
    pub time: UtcDateTime,
    pub data: Vec<BlockData>,
    pub previous_hash: Hash,
    pub signature: Signature,
    pub hash: Hash,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BlockChain {
    chain: Vec<Block>,
}

impl BlockData {
    pub fn hash(&self) -> Hash {
        use sha3::{Digest, Sha3_256};

        let mut hasher = Sha3_256::new();

        match self {
            Self::CreateWallet { id, key, time, .. } => {
                let serialized = serde_json::json!({
                    "id": id,
                    "key": key,
                    "time": time,
                });

                hasher.update(serialized.to_string().as_bytes());
            }
            Self::Transaction {
                fee,
                amount,
                time,
                sender,
                receiver,
                ..
            } => {
                let serialized = serde_json::json!({
                    "fee": fee,
                    "amount": amount,
                    "time": time,
                    "sender": sender,
                    "receiver": receiver,
                });

                hasher.update(serialized.to_string().as_bytes());
            }
        }

        Hash::new(hasher.finalize().to_vec())
    }
}

impl Block {
    pub fn hash(&self) -> Hash {
        use sha3::{Digest, Sha3_256};

        let mut hasher = Sha3_256::new();

        let serialized = serde_json::json!({
            "id": self.id,
            "nonce": self.nonce,
            "miner": self.miner,
            "time": self.time,
            "data": self.data,
            "previous_hash": self.previous_hash,
        });

        hasher.update(serialized.to_string().as_bytes());

        Hash::new(hasher.finalize().to_vec())
    }

    pub fn signed_hash(&self) -> Hash {
        use sha3::{Digest, Sha3_256};

        let mut hasher = Sha3_256::new();

        let serialized = serde_json::json!({
            "id": self.id,
            "nonce": self.nonce,
            "miner": self.miner,
            "time": self.time,
            "data": self.data,
            "previous_hash": self.previous_hash,
            "signature": self.signature
        });

        hasher.update(serialized.to_string().as_bytes());

        Hash::new(hasher.finalize().to_vec())
    }
}

impl BlockChain {
    pub const HASH_DIFFICULTY: &[u8] = &[0xFC];

    pub fn new() -> Self {
        Self { chain: Vec::new() }
    }

    pub fn blocks(&self) -> &[Block] {
        &self.chain
    }

    pub fn add_block(&mut self, block: Block) {
        self.chain.push(block);
    }

    pub fn mine_block(block: &mut Block, key: &PrivateKey) -> bool {
        block.hash = loop {
            let Some(signature) = key.sign(&block.hash()) else {
                return false;
            };

            block.signature = signature;

            let hash = block.signed_hash();

            if hash.bytes().starts_with(Self::HASH_DIFFICULTY) {
                break hash;
            }

            block.nonce += 1;
        };

        true
    }

    pub fn mine_block_with_rng(
        mut rng: impl rand_core::CryptoRng + rand_core::RngCore,
        block: &mut Block,
        key: &PrivateKey,
    ) -> bool {
        block.hash = loop {
            let Some(signature) = key.sign_with_rng(&mut rng, &block.hash()) else {
                return false;
            };

            block.signature = signature;

            let hash = block.signed_hash();

            if hash.bytes().starts_with(Self::HASH_DIFFICULTY) {
                break hash;
            }

            block.nonce += 1;
        };

        true
    }

    pub fn validate(&self) -> bool {
        for i in 1..self.chain.len() {
            let a = &self.chain[i - 1];
            let b = &self.chain[i];

            if b.id != a.id + 1
                || a.hash != a.signed_hash()
                || !a.hash.bytes().starts_with(Self::HASH_DIFFICULTY)
                || b.hash != b.signed_hash()
                || !b.hash.bytes().starts_with(Self::HASH_DIFFICULTY)
                || b.previous_hash != a.hash
                || !a.miner.verify(&a.hash(), &a.signature)
                || !b.miner.verify(&b.hash(), &b.signature)
            {
                return false;
            }
        }

        true
    }
}
