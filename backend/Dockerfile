FROM rust:1.86.0-slim-bookworm as base

FROM base as builder
WORKDIR /app  
COPY ./backend ./backend
COPY ./shared ./shared
WORKDIR /app/backend  
RUN cargo build --release --bin backend
RUN strip ./target/release/backend -o /backend

FROM debian:bullseye-slim
COPY --from=builder /backend /
COPY ./data ./data

EXPOSE 4200

CMD [ "/backend" ]