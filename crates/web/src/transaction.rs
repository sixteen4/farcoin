use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension, Json,
};
use farcoin::{BlockData, Hash, PrivateKey, PublicKey, UtcDateTime};
use serde::{Deserialize, Serialize};

use crate::WorldHandle;

pub async fn get() -> Html<&'static str> {
    Html(include_str!("../../../frontend/transaction.html"))
}

#[derive(Debug, Deserialize)]
pub struct Request {
    receiver_key: String,
    public_key: String,
    private_key: String,
    amount: String,
    fee: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
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

    let receiver_key = if let Ok(receiver_key) = PublicKey::try_from(request.receiver_key.as_str())
    {
        receiver_key
    } else if let Ok(student_id) = request.receiver_key.parse::<u64>() {
        if let Some(receiver_key) = world.wallet_ids.get(&student_id) {
            receiver_key.clone()
        } else {
            return (
                StatusCode::CREATED,
                Json(Response {
                    valid: false,
                    message: "Student not found!".into(),
                }),
            );
        }
    } else {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Invalid receiver key!".into(),
            }),
        );
    };

    if !world.wallets.contains_key(&receiver_key) {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Receiver wallet does not exist!".into(),
            }),
        );
    }

    let public_key = if let Ok(public_key) = PublicKey::try_from(request.public_key.as_str()) {
        public_key
    } else if let Ok(student_id) = request.public_key.parse::<u64>() {
        if let Some(public_key) = world.wallet_ids.get(&student_id) {
            public_key.clone()
        } else {
            return (
                StatusCode::CREATED,
                Json(Response {
                    valid: false,
                    message: "Student not found!".into(),
                }),
            );
        }
    } else {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Invalid public key!".into(),
            }),
        );
    };

    if !world.wallets.contains_key(&public_key) {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Wallet does not exist!".into(),
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
                message: "Invalid public key or student ID!".into(),
            }),
        );
    }

    let Ok(amount) = request.amount.parse::<u64>() else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Invalid amount!".into() }));
    };

    if amount == 0 {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Amount must be greater than zero!".into(),
            }),
        );
    }

    let Ok(fee) = request.fee.parse::<u64>() else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Invalid fee!".into() }));
    };

    if fee >= amount {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Fee must be less than the amount!".into(),
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

    let mut transaction = BlockData::Transaction {
        fee,
        amount,
        time,
        sender: public_key.clone(),
        receiver: receiver_key.clone(),
        signature,
    };

    let transaction_hash = transaction.hash();

    let BlockData::Transaction { ref mut signature, .. } = transaction else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Internal Server Error".into() }));
    };

    let Some(transaction_signature) = private_key.sign_with_rng(&mut rng, &transaction_hash) else {
        return (StatusCode::CREATED, Json(Response { valid: false, message: "Internal Server Error".into() }));
    };

    *signature = transaction_signature;

    if !world.verify_data(&transaction) {
        return (
            StatusCode::CREATED,
            Json(Response {
                valid: false,
                message: "Invalid request!".into(),
            }),
        );
    }

    world.waiting.push(transaction);

    (
        StatusCode::CREATED,
        Json(Response {
            valid: true,
            message: "Successfully made transaction!".into(),
        }),
    )
}
