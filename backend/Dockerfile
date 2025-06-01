# Build stage
FROM rust:1.75-slim-bullseye as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/messaging_app .

# Create a non-root user
RUN useradd -m -u 1000 app

# Switch to non-root user
USER app

# Set environment variables
ENV RUST_LOG=info
ENV APP_ENV=production

# Expose the port
EXPOSE 3000

# Run the application
CMD ["./messaging_app"] 