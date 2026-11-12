use axum::{Json, Router, routing::get};
use serde::Serialize;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("xyd is listening on port 3000");

    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct RootResponse {
    message: String,
}

async fn root() -> Json<RootResponse> {
    let response = RootResponse {
        message: "hello from xyd".to_string(),
    };

    Json(response)
}
