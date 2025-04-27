FROM rust:1.81.0-slim-bullseye as base

FROM base as builder
WORKDIR /app
COPY ./backend ./backend
COPY ./shared ./shared
RUN cargo build --release --bin backend
RUN strip ./target/release/backend -o /backend

FROM debian:bullseye-slim
COPY --from=builder /backend /
COPY ./data ./data

EXPOSE 4200

CMD [ "/backend" ]