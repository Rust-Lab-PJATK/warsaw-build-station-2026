FROM rust:1.88-slim AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config
COPY assets ./assets

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV RUST_LOG=info
ENV LOCO_ENV=development

COPY --from=builder /app/target/release/backend-cli /app/backend
COPY --from=builder /app/config /app/config
COPY --from=builder /app/assets /app/assets

EXPOSE 5150

CMD ["/app/backend", "start"]
