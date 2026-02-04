---
title: "Error Handling in Rust: Result, Option, and the ? Operator"
date: 2026-02-03
tags: [rust, error-handling, programming]
summary: "Rust replaces exceptions with algebraic types. Here's how Result, Option, and the ? operator make error handling explicit without being verbose."
---

Most languages handle errors with exceptions — invisible control flow that can jump anywhere. Rust takes a different approach: errors are values, and you must handle them explicitly.

## Option: Something or Nothing

`Option<T>` represents a value that might not exist:

```rust
fn find_user(id: u32) -> Option<User> {
    users.iter().find(|u| u.id == id).cloned()
}

match find_user(42) {
    Some(user) => println!("Found: {}", user.name),
    None => println!("User not found"),
}
```

No null pointer exceptions. The compiler forces you to handle the `None` case.

## Result: Success or Failure

`Result<T, E>` represents an operation that can fail:

```rust
use std::fs;
use std::io;

fn read_config(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

match read_config("config.toml") {
    Ok(content) => parse(content),
    Err(e) => eprintln!("Failed to read config: {}", e),
}
```

The error type `E` is explicit. You know exactly what can go wrong and can handle each case.

## The ? Operator

Writing `match` for every fallible operation is tedious. The `?` operator propagates errors automatically:

```rust
fn setup() -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&content)?;
    let db = Database::connect(&config.db_url)?;
    Ok(config)
}
```

Each `?` says: "if this is `Err`, return it immediately. Otherwise, unwrap the `Ok` value." It's like exceptions but visible — every `?` marks a potential early return.

## Custom Error Types

For libraries and larger applications, define your own error type:

```rust
#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    NotFound(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Parse(e)
    }
}
```

With `From` implementations, the `?` operator automatically converts between error types.

## thiserror and anyhow

For production code, two crates simplify error handling:

**thiserror** — derive macro for custom error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("resource not found: {0}")]
    NotFound(String),
}
```

**anyhow** — when you don't need typed errors:

```rust
use anyhow::{Context, Result};

fn setup() -> Result<()> {
    let config = fs::read_to_string("config.toml")
        .context("failed to read config file")?;
    Ok(())
}
```

## Compared to Exceptions

| | Exceptions | Result/Option |
|---|---|---|
| Visibility | Hidden control flow | Explicit in types |
| Performance | Stack unwinding overhead | Zero-cost (no unwinding) |
| Compiler help | None | Forces handling |
| Composition | try/catch nesting | `?` chaining |

The tradeoff is verbosity for safety. In practice, `?` makes the verbosity minimal, and the compiler catches every unhandled error before your code runs.
