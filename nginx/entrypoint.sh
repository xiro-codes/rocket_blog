#!/bin/sh

# Check if SSL certificates exist
if [ ! -f "/etc/letsencrypt/live/blog.tdavis.dev/fullchain.pem" ]; then
    echo "SSL certificates not found. Starting nginx with HTTP only..."
    echo "Please run the SSL setup script to generate certificates."
    
    # Create a temporary HTTP-only nginx config
    cat > /etc/nginx/nginx.conf << 'EOF'
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log;
pid /run/nginx.pid;

events {
    worker_connections 1024;
}

http {
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;

    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;

    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    server {
        listen 80;
        server_name blog.tdavis.dev;
        
        # Allow certbot challenges
        location /.well-known/acme-challenge/ {
            root /var/www/certbot;
        }
        
        # Proxy to app for now (until SSL is set up)
        location / {
            proxy_pass http://app:8000;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
EOF
fi

# Start nginx
nginx -g "daemon off;"