FROM rust:1 AS builder
WORKDIR /usr/src/bandwidth-hero-proxy-rs
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/bandwidth-hero-proxy-rs /usr/local/bin/bandwidth-hero-proxy-rs
CMD ["bandwidth-hero-proxy-rs"]
    