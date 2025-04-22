FROM rust:1.81.0-slim-bullseye as base

FROM base as builder
WORKDIR /app
COPY ./backend /app/backend
COPY ./shared /app/shared
RUN cargo build --release --bin backend
RUN strip /app/target/release/backend -o /backend

FROM debian:bullseye-slim
# COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.5.0 /lambda-adapter /opt/extensions/lambda-adapter
COPY --from=builder /backend /
COPY ./data /app/data

EXPOSE 4200

CMD [ "/backend" ]