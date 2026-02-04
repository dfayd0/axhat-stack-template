FROM rust:1.84-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY build.rs ./
COPY templates ./templates
COPY public ./public

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/portfolio .
COPY --from=builder /app/public ./public
COPY --from=builder /app/templates ./templates

EXPOSE 4444

CMD ["./portfolio"]
