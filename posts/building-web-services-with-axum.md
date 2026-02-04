---
title: "Building Web Services with Axum"
date: 2026-01-28
tags: [rust, axum, web, backend]
summary: "How to build fast, type-safe web services in Rust using Axum — from routing to middleware to error handling."
---

Axum is a web framework built on top of Tower and Hyper. It's fast, ergonomic, and leverages Rust's type system to catch mistakes at compile time rather than runtime.

## Why Axum

There are several Rust web frameworks — Actix Web, Rocket, Warp. I chose Axum for a few reasons:

- **Tower ecosystem**: middleware, load balancing, timeouts — all composable
- **Type-safe extractors**: request parsing is checked at compile time
- **No macros for routing**: routes are plain Rust, easy to refactor
- **Active development**: maintained by the Tokio team

## A Minimal Server

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

That's it. No boilerplate, no configuration files. The handler is an async function that returns something implementing `IntoResponse`.

## Extractors

Extractors are how you pull data from requests. They're type-safe and composable:

```rust
use axum::{extract::{Path, Query, Json}, response::Json as JsonResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

async fn get_user(
    Path(user_id): Path<u32>,
    Query(pagination): Query<Pagination>,
) -> JsonResponse<User> {
    // user_id and pagination are already parsed and validated
    let user = fetch_user(user_id, pagination).await;
    JsonResponse(user)
}
```

If the path parameter isn't a valid `u32`, or the query string is malformed, Axum returns a 400 error automatically. No manual parsing or validation needed.

## Shared State

Application state is injected via extractors too:

```rust
use axum::extract::State;
use std::sync::Arc;

struct AppState {
    db: DatabasePool,
}

async fn handler(State(state): State<Arc<AppState>>) -> String {
    let count = state.db.get_user_count().await;
    format!("Total users: {}", count)
}

let app = Router::new()
    .route("/", get(handler))
    .with_state(Arc::new(AppState { db: pool }));
```

## Error Handling

Axum handlers can return `Result` types. Combined with a custom error type, you get structured error responses:

```rust
use axum::{http::StatusCode, response::IntoResponse};

enum AppError {
    NotFound,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }
}
```

## Middleware with Tower

Since Axum is built on Tower, you get the entire Tower middleware ecosystem:

```rust
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};

let app = Router::new()
    .route("/api/data", get(handler))
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new())
    .layer(CorsLayer::permissive());
```

Each layer is independent and composable. You can apply middleware to specific route groups, not just globally.

## Performance

Axum on Tokio handles tens of thousands of concurrent connections with minimal overhead. The framework adds almost nothing on top of raw Hyper — most of the "framework" is compile-time type machinery that disappears in the binary.

For this portfolio site, it's admittedly overkill. But when you need to scale from a simple site to a high-throughput API, you don't need to switch frameworks.
