#!/bin/sh

# Check if SSL certificates exist
if [ ! -f "/etc/letsencrypt/live/blog.tdavis.dev/fullchain.pem" ]; then
    echo "SSL certificates not found. Starting nginx with HTTP only..."
    echo "Please run the SSL setup script to generate certificates."
    echo "Creating HTTP-only nginx configuration..."
    
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
            # Use a resolver to allow nginx to start even if app is not available
            resolver 127.0.0.11 valid=30s;
            set $upstream app:8000;
            
            proxy_pass http://$upstream;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            
            # File upload support
            client_max_body_size 1G;
            
            # Handle backend errors gracefully
            proxy_connect_timeout 5s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
            
            # Custom error pages for when app is not available
            error_page 502 503 504 /50x.html;
        }
        
        # Error page for when app is not available
        location = /50x.html {
            return 200 "Blog is starting up... Please wait a moment and refresh.";
            add_header Content-Type text/plain;
        }
    }

    # Work Time Tracker HTTP server
    server {
        listen 80;
        server_name worktime.tdavis.dev;
        
        # Allow certbot challenges
        location /.well-known/acme-challenge/ {
            root /var/www/certbot;
        }
        
        # Proxy to worktime app for now (until SSL is set up)
        location / {
            # Use a resolver to allow nginx to start even if app is not available
            resolver 127.0.0.11 valid=30s;
            set $upstream worktime:8001;
            
            proxy_pass http://$upstream;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            
            # File upload support
            client_max_body_size 1G;
            
            # Handle backend errors gracefully
            proxy_connect_timeout 5s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
            
            # Custom error pages for when app is not available
            error_page 502 503 504 /50x.html;
        }
        
        # Error page for when app is not available
        location = /50x.html {
            return 200 "Work Time Tracker is starting up... Please wait a moment and refresh.";
            add_header Content-Type text/plain;
        }
    }
}
EOF
    echo "HTTP-only configuration created successfully."
else
    echo "SSL certificates found. Using full SSL configuration."
    cp /etc/nginx/nginx.ssl.conf /etc/nginx/nginx.conf
fi

# Verify nginx configuration syntax
echo "Testing nginx configuration..."
if nginx -t; then
    echo "Nginx configuration is valid."
else
    echo "ERROR: Nginx configuration is invalid!"
    exit 1
fi

# Start nginx
echo "Starting nginx..."
nginx -g "daemon off;"