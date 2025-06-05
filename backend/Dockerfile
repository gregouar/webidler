FROM rust:1.86.0-slim-bookworm as base

FROM base as builder
WORKDIR /app  
COPY ./backend ./backend
COPY ./shared ./shared
WORKDIR /app/backend  
RUN cargo build --release --no-default-features --bin backend --features postgres 
RUN strip ./target/release/backend -o /backend

FROM debian:bookworm-slim
COPY --from=builder /backend /

RUN addgroup --system appgroup && adduser --system --ingroup appgroup appuser
RUN chown appuser:appgroup /backend -R

EXPOSE 4200

USER appuser

CMD [ "/backend" ]