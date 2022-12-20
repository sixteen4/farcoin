use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension, Json,
};
use serde::Serialize;

use crate::WorldHandle;

pub async fn get() -> Html<&'static str> {
    Html(include_str!("../../../frontend/validate.html"))
}

#[derive(Debug, Serialize)]
struct Response {
    status: String,
    chain_data: String,
}

pub async fn post(Extension(world): Extension<WorldHandle>) -> impl IntoResponse {
    let Ok(world) = world.lock() else {
        return (StatusCode::CREATED, Json(Response { status: "Internal Server Error".to_string(), chain_data: "".to_string() }));
    };

    (
        StatusCode::CREATED,
        Json(Response {
            status: world
                .chain
                .validate()
                .then_some("Valid".to_string())
                .unwrap_or("Invalid".to_string()),
            chain_data: serde_json::to_string_pretty(&world.chain)
                .unwrap_or("Internal Server Error".to_string()),
        }),
    )
}
