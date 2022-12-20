use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension, Json,
};
use farcoin::{BlockData, Hash, PrivateKey, PublicKey, UtcDateTime};
use serde::{Deserialize, Serialize};

use crate::WorldHandle;

pub async fn get() -> Html<&'static str> {
    Html(include_str!("../../../frontend/wallet.html"))
}

#[derive(Debug, Deserialize)]
pub struct Request {
    student_id: String,
    public_key: String,
    private_key: String,
}

#[derive(Debug, Serialize)]
struct Response {
    valid: bool,
    message: String,
}

pub async fn post(
    Json(request): Json<Request>,
    Extension(world): Extension<WorldHandle>,
) -> impl IntoResponse {
    let Ok(mut world) = world.lock() else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Internal Server Error".into() }));
    };

    let Ok(student_id) = request.student_id.parse::<u64>() else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Invalid student ID!".into() }));
    };

    if world.wallet_ids.contains_key(&student_id) {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Student ID already in use!".into(),
            }),
        );
    }

    let Ok(public_key) = PublicKey::try_from(request.public_key.as_str()) else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Invalid public key!".into() }));
    };

    if world.wallets.contains_key(&public_key) {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Public key already in use!".into(),
            }),
        );
    }

    let Ok(private_key) = PrivateKey::try_from(request.private_key.as_str()) else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Invalid private key!".into() }));
    };

    if public_key != PublicKey::from(&private_key) {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Invalid public key!".into(),
            }),
        );
    }

    let Some(time) = UtcDateTime::now() else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Internal Server Error".into() }));
    };

    let mut rng = rand::thread_rng();

    let Some(signature) = private_key.sign_with_rng(&mut rng, &Hash::empty()) else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Internal Server Error".into() }));
    };

    let mut wallet = BlockData::CreateWallet {
        id: student_id,
        key: public_key,
        time,
        signature,
    };

    let wallet_hash = wallet.hash();

    let BlockData::CreateWallet { ref mut signature, .. } = wallet else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Internal Server Error".into() }));
    };

    let Some(wallet_signature) = private_key.sign_with_rng(&mut rng, &wallet_hash) else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Internal Server Error".into() }));
    };

    *signature = wallet_signature;

    if !world.verify_data(&wallet) {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Invalid request!".into(),
            }),
        );
    }

    world.waiting.push(wallet);

    (
        StatusCode::CREATED,
        Json(Response {
            valid: true,
            message: "Successfully created new wallet!".into(),
        }),
    )
}
