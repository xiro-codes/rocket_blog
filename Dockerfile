# Use a Debian slim base image for consistency
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
COPY Rocket.docker.toml ./Rocket.toml

# For development, expect binary to be mounted or copied
# This allows building on host and running in container
COPY target/release/rocket-template ./rocket-template

# Create data directory for file uploads
RUN mkdir -p /app/data && chown -R app:app /app

# Ensure binary is executable if it exists
RUN if [ -f ./rocket-template ]; then chmod +x ./rocket-template; fi

# Change to app user
USER app

# Expose port
EXPOSE 8000

# Run the application
CMD ["./rocket-template"]