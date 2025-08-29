# Alternative runtime Dockerfile for dual-binary architecture
# This expects binaries to be built on host and copied/mounted into container

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    patchelf \
    ca-certificates \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install yt-dlp for youtube download functionality
RUN pip3 install --no-cache-dir --break-system-packages yt-dlp

# Create app user
RUN useradd -m -u 1000 app

# Set working directory
WORKDIR /app

# Copy static assets and templates
COPY templates ./templates/
COPY static ./static/
COPY scripts/docker/.Rocket.docker.toml ./Rocket.toml

# For development, expect binaries to be mounted or copied
# This allows building on host and running in container
COPY target/release/blog ./blog
COPY target/release/worktime ./worktime

# Create data directory for file uploads
RUN mkdir -p /app/data && chown -R app:app /app

# Ensure binaries are executable if they exist
RUN if [ -f ./blog ]; then chmod +x ./blog; fi && \
    if [ -f ./worktime ]; then chmod +x ./worktime; fi

# Patch executables to use the correct dynamic linker
RUN LD_PATH=$(find / -name "ld-linux-x86-64.so.2") && \
    if [ -f ./blog ]; then patchelf --set-interpreter "$LD_PATH" ./blog; fi && \
    if [ -f ./worktime ]; then patchelf --set-interpreter "$LD_PATH" ./worktime; fi

# Change to app user
USER app

# Expose ports for both services
EXPOSE 8000 8001

# Default to blog binary (can be overridden)
CMD ["./blog"]
