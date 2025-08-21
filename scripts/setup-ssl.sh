#!/bin/bash

# This script helps with the initial setup of SSL certificates
# Run this BEFORE starting the full stack if you don't have certificates yet

echo "Setting up initial SSL certificates for blog.tdavis.dev..."

# Check if nginx is already running and stop it temporarily
echo "Stopping any running nginx containers..."
docker compose -f scripts/docker/docker-compose.yml stop nginx 2>/dev/null || true

# Start nginx in HTTP-only mode for certificate generation
echo "Starting nginx in HTTP-only mode..."
docker compose -f scripts/docker/docker-compose.yml up -d nginx

# Wait for nginx to start
echo "Waiting for nginx to start..."
sleep 10

# Generate certificates using standalone certbot
echo "Generating SSL certificates..."
docker compose -f scripts/docker/docker-compose.yml run --rm certbot \
    certonly \
    --webroot \
    --webroot-path=/var/www/certbot \
    --email me@tdavis.dev \
    --agree-tos \
    --no-eff-email \
    --force-renewal \
    -d blog.tdavis.dev

# Check if certificate generation was successful
if [ $? -eq 0 ]; then
    echo "SSL certificates generated successfully!"
    
    # Restart nginx with the full SSL configuration
    echo "Restarting nginx with SSL configuration..."
    docker compose -f scripts/docker/docker-compose.yml stop nginx
    
    # Copy the full SSL configuration back
    docker compose -f scripts/docker/docker-compose.yml up -d nginx
    
    echo ""
    echo "SSL setup complete!"
    echo "You can now access your site at: https://blog.tdavis.dev"
else
    echo "SSL certificate generation failed!"
    echo "Please check your DNS settings and ensure blog.tdavis.dev points to this server."
    exit 1
fi