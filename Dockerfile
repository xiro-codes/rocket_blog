# Multi-stage Docker build for Rocket Blog
# Stage 1: Build the application using Ubuntu base instead of Debian
FROM ubuntu:22.04 AS builder

# Prevent timezone prompt during apt install
ENV DEBIAN_FRONTEND=noninteractive

# Install Rust and dependencies 
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and verify installation
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.89.0 && \
    /root/.cargo/bin/cargo --version
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations/
COPY models ./models/

# Copy source code
COPY src ./src/

# Build the application in release mode
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
COPY .Rocket.docker.toml ./Rocket.toml

# Copy the built binary from builder stage
# Note: Cargo.toml defines the binary name as "app"
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