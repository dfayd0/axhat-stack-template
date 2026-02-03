use axum::{routing::get, Router};

use crate::routes::*;
mod routes;
mod templates;

#[tokio::main]
async fn main()
{
    tracing_subscriber::fmt::init();
    let address = "0.0.0.0".to_owned();
    let port = "4444";
    let app = Router::new()
        .route("/", get(index))
        .fallback(error_handler)
        .nest("/public", routes::serve_static_files());

    let listener = tokio::net::TcpListener::bind(address + ":" + &port)
        .await
        .expect("Failed to bind port.");
    println!("Server running on http://{}:{}", "localhost", port);

    axum::serve(listener, app).await.unwrap();
}
