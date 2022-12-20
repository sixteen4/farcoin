mod balance;
mod index;
mod transaction;
mod validate;
mod wallet;
mod world;

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    routing::{get, post},
    Extension, Router,
};
use axum_extra::routing::SpaRouter;
use farcoin::{Block, BlockChain, BlockData, Hash, PrivateKey, PublicKey, UtcDateTime};
use tokio::time::sleep;
use world::WorldState;

pub type WorldHandle = Arc<Mutex<WorldState>>;

async fn mine_worker(handle: WorldHandle, genesis_keys: (PublicKey, PrivateKey)) {
    let mut rng = rand::thread_rng();

    let empty_signature = genesis_keys
        .1
        .sign_with_rng(&mut rng, &Hash::empty())
        .expect("signed empty hash");

    loop {
        sleep(Duration::from_millis(5000)).await;

        let Ok(mut world) = handle.lock() else {
            println!("Failed to lock world!");
            continue;
        };

        if world.waiting.len() > 0 {
            println!("Processing {} events...", world.waiting.len());

            let (id, previous_hash) = if let Some(last) = world.chain.blocks().last() {
                (last.id + 1, last.hash.clone())
            } else {
                (0, Hash::empty())
            };

            let Some(time) = UtcDateTime::now() else {
                println!("Failed to obtain time!");
                continue;
            };

            let mut block = Block {
                id,
                nonce: 0,
                miner: genesis_keys.0.clone(),
                time,
                data: world.waiting.clone(),
                previous_hash,
                signature: empty_signature.clone(),
                hash: Hash::empty(),
            };

            world.waiting.clear();

            if !BlockChain::mine_block_with_rng(&mut rng, &mut block, &genesis_keys.1) {
                println!("Failed to mine block!");
                continue;
            }

            if !world.add_block(block) {
                println!("Failed to validate new block!");
                continue;
            }

            println!("Complete!");
        }
    }
}

#[tokio::main]
async fn main() {
    let mut rng = rand::thread_rng();

    println!("Generating genesis keys...");

    let genesis_key = PrivateKey::random(&mut rng);
    let public_key = PublicKey::from(&genesis_key);

    let empty_signature = genesis_key.sign(&Hash::empty()).unwrap();

    println!("Creating block chain...");

    let mut chain = BlockChain::new();

    let mut genesis_block = Block {
        id: 0,
        nonce: 0,
        miner: public_key.clone(),
        time: UtcDateTime::now().unwrap(),
        data: vec![],
        previous_hash: Hash::empty(),
        signature: empty_signature.clone(),
        hash: Hash::empty(),
    };

    println!("Mining genesis block...");

    if !BlockChain::mine_block_with_rng(&mut rng, &mut genesis_block, &genesis_key) {
        panic!("Failed to mine genesis block!");
    }

    chain.add_block(genesis_block);

    let mut test_wallet = BlockData::CreateWallet {
        id: 0,
        key: public_key.clone(),
        time: UtcDateTime::now().unwrap(),
        signature: empty_signature.clone(),
    };

    let test_wallet_hash = test_wallet.hash();

    let BlockData::CreateWallet { ref mut signature, .. } = test_wallet else {
        panic!("what");
    };

    *signature = genesis_key
        .sign_with_rng(&mut rng, &test_wallet_hash)
        .unwrap();

    let mut test_block = Block {
        id: 1,
        nonce: 0,
        miner: public_key.clone(),
        time: UtcDateTime::now().unwrap(),
        data: vec![test_wallet],
        previous_hash: chain.blocks().last().unwrap().hash.clone(),
        signature: empty_signature.clone(),
        hash: Hash::empty(),
    };

    if !BlockChain::mine_block_with_rng(&mut rng, &mut test_block, &genesis_key) {
        panic!("Failed to mine test block!");
    }

    chain.add_block(test_block);

    let world_state = WorldHandle::new(Mutex::new(
        WorldState::new(chain).expect("valid world state"),
    ));

    println!("Server started!");

    tokio::join!(
        async {
            let app = Router::new()
                .route("/", get(index::get))
                .route("/transaction", get(transaction::get))
                .route("/transaction", post(transaction::post))
                .route("/balance", get(balance::get))
                .route("/balance", post(balance::post))
                .route("/wallet", get(wallet::get))
                .route("/wallet", post(wallet::post))
                .route("/validate", get(validate::get))
                .route("/validate", post(validate::post))
                .layer(Extension(world_state.clone()))
                .merge(SpaRouter::new("/assets", "frontend/assets").index_file("error.html"));

            let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        },
        mine_worker(world_state.clone(), (public_key, genesis_key))
    );
}
