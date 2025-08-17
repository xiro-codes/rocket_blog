FROM nginx:alpine

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Create directories for certbot
RUN mkdir -p /var/www/certbot
RUN mkdir -p /etc/letsencrypt

# Expose ports
EXPOSE 80 443

# Use custom entrypoint
ENTRYPOINT ["/entrypoint.sh"]