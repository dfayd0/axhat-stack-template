FROM rust:slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY build.rs ./
COPY templates ./templates
COPY public ./public
COPY posts ./posts

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/portfolio .
COPY public ./public
COPY templates ./templates
COPY posts ./posts

EXPOSE 4444

CMD ["./portfolio"]
