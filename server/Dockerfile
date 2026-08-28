FROM rust:1-slim-bookworm AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/vigil-server ./vigil-server
COPY --from=builder /app/migrations ./migrations
EXPOSE 8080
CMD ["./vigil-server"]
