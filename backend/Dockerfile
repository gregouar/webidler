FROM rust:1.81.0-slim-bullseye as base

FROM base as builder
WORKDIR /app
COPY ./backend /app/backend
COPY ./shared /app/shared
RUN cargo build --release --bin backend
RUN strip /app/target/release/backend -o /backend

FROM debian:bullseye-slim
COPY --from=builder /backend /
COPY ./data /app/data

EXPOSE 4200

CMD [ "/backend" ]