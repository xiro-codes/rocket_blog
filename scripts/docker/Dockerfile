# Multi-stage Docker build for Rocket Blog
# This Dockerfile builds the application from source, solving the NixOS build issue

# Stage 1: Build the application
FROM rust:1.89-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better Docker layer caching
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations/
COPY models ./models/

# Copy source code
COPY src ./src/

# Build the application in release mode
# Note: If you encounter SSL certificate issues during build,
# see the README.md for alternative build approaches
RUN cargo build --release

# Stage 2: Runtime image  
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 app

# Set working directory
WORKDIR /app

# Copy static assets and templates
COPY templates ./templates/
COPY static ./static/
COPY scripts/docker/.Rocket.docker.toml ./Rocket.toml

# Copy the built binary from builder stage
# Note: The binary is named "app" in Cargo.toml but we rename it to rocket-template for consistency
COPY --from=builder /app/target/release/app ./rocket-template

# Create data directory for file uploads
RUN mkdir -p /app/data && chown -R app:app /app

# Ensure binary is executable
RUN chmod +x ./rocket-template

# Change to app user
USER app

# Expose port
EXPOSE 8000

# Run the application
CMD ["./rocket-template"]