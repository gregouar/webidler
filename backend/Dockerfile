FROM rust:1.89.0-slim-bookworm AS base

FROM base AS builder
WORKDIR /app  
COPY ./backend ./backend
COPY ./shared ./shared
COPY ./.sqlx ./backend/.sqlx

WORKDIR /app/backend  

ENV SQLX_OFFLINE=true
RUN cargo build --release --no-default-features --bin backend --features postgres 
RUN strip ./target/release/backend -o /backend

FROM debian:bookworm-slim
COPY --from=builder /backend /
COPY ./migrations ./migrations

RUN addgroup --system appgroup && adduser --system --ingroup appgroup appuser
RUN chown appuser:appgroup /backend -R

EXPOSE 4200

USER appuser

CMD [ "/backend" ]