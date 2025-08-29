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

# Build the application in release mode for both binaries
# Note: If you encounter SSL certificate issues during build,
# see the README.md for alternative build approaches
RUN cargo build --release --bin blog --bin worktime

# Stage 2: Runtime image  
FROM debian:bookworm-slim

# Install runtime dependencies including Python and yt-dlp
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install yt-dlp using pip (break system packages restriction in container environment)
RUN pip3 install --no-cache-dir --break-system-packages yt-dlp

# Create app user
RUN useradd -m -u 1000 app

# Set working directory
WORKDIR /app

# Copy static assets and templates
COPY templates ./templates/
COPY static ./static/
COPY scripts/docker/.Rocket.docker.toml ./Rocket.toml

# Copy the built binaries from builder stage
COPY --from=builder /app/target/release/blog ./blog
COPY --from=builder /app/target/release/worktime ./worktime

# Create data directory for file uploads
RUN mkdir -p /app/data && chown -R app:app /app

# Ensure binaries are executable
RUN chmod +x ./blog ./worktime

# Change to app user
USER app

# Expose ports for both services
EXPOSE 8000 8001

# Run the blog application by default (can be overridden)
CMD ["./blog"]