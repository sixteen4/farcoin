use axum::response::Html;

pub async fn get() -> Html<&'static str> {
    Html(include_str!("../../../frontend/index.html"))
}
