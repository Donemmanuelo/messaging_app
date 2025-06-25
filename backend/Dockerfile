# syntax=docker/dockerfile:1
FROM rust:1.76 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/messaging_app_backend /usr/local/bin/messaging_app_backend
COPY .env .
EXPOSE 8080
CMD ["/usr/local/bin/messaging_app_backend"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 CMD curl -f http://localhost:8080/health || exit 1 