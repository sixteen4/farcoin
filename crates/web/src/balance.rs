use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension, Json,
};
use farcoin::PublicKey;
use serde::{Deserialize, Serialize};

use crate::WorldHandle;

pub async fn get() -> Html<&'static str> {
    Html(include_str!("../../../frontend/balance.html"))
}

#[derive(Debug, Deserialize)]
pub struct Request {
    public_key: String,
}

#[derive(Debug, Serialize)]
struct Response {
    balance: String,
}

pub async fn post(
    Json(request): Json<Request>,
    Extension(world): Extension<WorldHandle>,
) -> impl IntoResponse {
    let Ok(world) = world.lock() else {
        return (StatusCode::CREATED, Json(Response { balance: "Internal Server Error".into() }));
    };

    let public_key = if let Ok(public_key) = PublicKey::try_from(request.public_key.as_str()) {
        public_key
    } else if let Ok(student_id) = request.public_key.parse::<u64>() {
        if let Some(public_key) = world.wallet_ids.get(&student_id) {
            public_key.clone()
        } else {
            return (
                StatusCode::CREATED,
                Json(Response {
                    balance: "Student not found!".into(),
                }),
            );
        }
    } else {
        return (
            StatusCode::CREATED,
            Json(Response {
                balance: "Invalid public key!".into(),
            }),
        );
    };

    let Some(wallet) = world.wallets.get(&public_key) else {
        return (StatusCode::CREATED, Json(Response { balance: "Wallet not found!".into() }));
    };

    (
        StatusCode::CREATED,
        Json(Response {
            balance: format!("{}", wallet.balance),
        }),
    )
}
