FROM nginx:alpine

# Copy nginx configurations as templates
COPY nginx.conf /etc/nginx/nginx.ssl.conf
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Create directories for certbot
RUN mkdir -p /var/www/certbot
RUN mkdir -p /etc/letsencrypt

# Expose ports
EXPOSE 80 443

# Use custom entrypoint
ENTRYPOINT ["/entrypoint.sh"]