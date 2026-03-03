FROM rust:1.91.0-slim-bookworm AS base

FROM base AS builder
WORKDIR /app  

# Install system dependencies for OpenSSL and pkg-config
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY ./backend_chat ./backend_chat
COPY ./backend_shared ./backend_shared
COPY ./shared_chat ./shared_chat

WORKDIR /app/backend_chat  

ENV SQLX_OFFLINE=true
RUN cargo build --release --no-default-features --bin backend_chat --features postgres 
RUN strip ./target/release/backend_chat -o /backend_chat

FROM debian:bookworm-slim

# Install system dependencies for OpenSSL and pkg-config
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*
 
COPY --from=builder /backend_chat /

RUN addgroup --system appgroup && adduser --system --ingroup appgroup appuser
RUN chown appuser:appgroup /backend_chat -R

EXPOSE 4242

USER appuser

CMD [ "/backend_chat" ]