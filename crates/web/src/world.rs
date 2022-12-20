use std::collections::HashMap;

use farcoin::{Block, BlockChain, BlockData, PublicKey, UtcDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transaction {
    Send {
        fee: u64,
        miner: PublicKey,
        amount: u64,
        receiver: PublicKey,
    },
    Receive {
        amount: u64,
        sender: PublicKey,
    },
    CollectFee {
        fee: u64,
        sender: PublicKey,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: u64,
    pub balance: u64,
    pub creation_time: UtcDateTime,
    pub transaction_history: HashMap<UtcDateTime, Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub chain: BlockChain,
    pub waiting: Vec<BlockData>,
    pub wallets: HashMap<PublicKey, Wallet>,
    pub wallet_ids: HashMap<u64, PublicKey>,
}

impl WorldState {
    pub fn verify_data(&self, data: &BlockData) -> bool {
        match data {
            farcoin::BlockData::CreateWallet {
                id, key, signature, ..
            } => {
                if self.wallet_ids.contains_key(&id) {
                    return false;
                }

                if self.wallets.contains_key(&key) {
                    return false;
                }

                if !key.verify(&data.hash(), signature) {
                    return false;
                }
            }
            farcoin::BlockData::Transaction {
                fee,
                amount,
                time,
                sender,
                receiver,
                signature,
            } => {
                if *amount == 0 {
                    return false;
                }

                if fee >= amount {
                    return false;
                }

                if !self.wallets.contains_key(&sender) {
                    return false;
                }

                if !self.wallets.contains_key(&receiver) {
                    return false;
                }

                if !sender.verify(&data.hash(), signature) {
                    return false;
                }

                let Some(sender_wallet) = self.wallets.get(&sender) else {
                    return false;
                };

                if sender_wallet.balance < *amount {
                    return false;
                }

                if sender_wallet.transaction_history.contains_key(&time) {
                    return false;
                }

                let Some(receiver_wallet) = self.wallets.get(&receiver) else {
                    return false;
                };

                if receiver_wallet.transaction_history.contains_key(&time) {
                    return false;
                }
            }
        }

        true
    }

    pub fn process_data(&mut self, data: &BlockData, block: &Block) -> bool {
        if !self.verify_data(data) {
            return false;
        }

        match data {
            farcoin::BlockData::CreateWallet { id, key, time, .. } => {
                if time >= &block.time {
                    return false;
                }

                self.wallets.insert(
                    key.clone(),
                    Wallet {
                        id: *id,
                        balance: 100,
                        creation_time: time.clone(),
                        transaction_history: HashMap::new(),
                    },
                );

                self.wallet_ids.insert(*id, key.clone());
            }
            farcoin::BlockData::Transaction {
                fee,
                amount,
                time,
                sender,
                receiver,
                ..
            } => {
                if time >= &block.time {
                    return false;
                }

                if !self.wallets.contains_key(&block.miner) {
                    return false;
                }

                let Some(sender_wallet) = self.wallets.get_mut(&sender) else {
                    return false;
                };

                sender_wallet.balance -= amount;

                sender_wallet.transaction_history.insert(
                    time.clone(),
                    Transaction::Send {
                        fee: *fee,
                        miner: block.miner.clone(),
                        amount: *amount,
                        receiver: receiver.clone(),
                    },
                );

                let Some(receiver_wallet) = self.wallets.get_mut(&receiver) else {
                    return false;
                };

                receiver_wallet.balance += amount - fee;

                receiver_wallet.transaction_history.insert(
                    time.clone(),
                    Transaction::Receive {
                        amount: amount - fee,
                        sender: sender.clone(),
                    },
                );

                let Some(miner_wallet) = self.wallets.get_mut(&block.miner) else {
                    return false;
                };

                miner_wallet.balance += fee;

                miner_wallet.transaction_history.insert(
                    time.clone(),
                    Transaction::CollectFee {
                        fee: *fee,
                        sender: sender.clone(),
                    },
                );
            }
        }

        true
    }

    pub fn add_block(&mut self, block: Block) -> bool {
        for data in &block.data {
            if !self.process_data(data, &block) {
                return false;
            }
        }

        self.chain.add_block(block);

        self.chain.validate()
    }

    pub fn new(chain: BlockChain) -> Option<Self> {
        if !chain.validate() {
            return None;
        }

        let mut world = Self {
            chain: BlockChain::new(),
            waiting: vec![],
            wallets: HashMap::new(),
            wallet_ids: HashMap::new(),
        };

        for block in chain.blocks() {
            for data in &block.data {
                if !world.process_data(data, block) {
                    return None;
                }
            }
        }

        world.chain = chain;

        Some(world)
    }
}
