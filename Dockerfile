FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

LABEL org.opencontainers.image.title="fping-rust" \
      org.opencontainers.image.authors="fping-rust Community" \
      org.opencontainers.image.description="High-performance ping tool written in Rust" \
      org.opencontainers.image.base.name="docker.io/library/alpine:latest" \
      org.opencontainers.image.source="https://github.com/gsnw/fping-rust" \
      org.opencontainers.image.homepage="https://www.gsnw.de/tools/fping-rust.php"

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/fping /usr/local/bin/fping

ENTRYPOINT ["fping"]