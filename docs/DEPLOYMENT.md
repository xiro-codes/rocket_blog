# Deployment Guide

This guide covers different methods to deploy the Rocket Blog application in production environments.

## 🚀 Quick Deployment Options

### 🐳 Docker Deployment (Recommended)

The easiest way to deploy Rocket Blog is using Docker and Docker Compose.

#### Prerequisites
- Docker 20.10+
- Docker Compose v2.0+

#### Steps

1. **Clone the Repository**
   ```bash
   git clone https://github.com/xiro-codes/rocket_blog.git
   cd rocket_blog
   ```

2. **Configure Environment**
   ```bash
   # Create production environment file
   cp .env.example .env
   
   # Edit .env file with your settings
   nano .env
   ```

3. **Build and Deploy**
   ```bash
   # Build and start all services
   just docker-prod
   # OR: docker-compose -f scripts/docker/docker-compose.yml up -d --build
   
   # Check service status
   just docker-status
   # OR: docker-compose -f scripts/docker/docker-compose.yml ps
   ```

4. **Initialize Database**
   ```bash
   # Run migrations
   just migrate
   # OR: docker-compose -f scripts/docker/docker-compose.yml exec app cargo run -p migrations
   ```

5. **Access Your Blog**
   - Blog: `http://your-domain.com:8000`
   - Admin Panel: `http://your-domain.com:5050` (pgAdmin)

### ☁️ Cloud Platform Deployment

#### Railway
```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and deploy
railway login
railway init
railway up
```

#### Render
1. Connect your GitHub repository
2. Set build command: `just build` or `cargo build --release`
3. Set start command: `./target/release/app`
4. Add PostgreSQL database
5. Configure environment variables

#### DigitalOcean App Platform
1. Create new app from GitHub repository
2. Configure build settings:
   - Build Command: `just build` or `cargo build --release`
   - Run Command: `./target/release/app`
3. Add PostgreSQL database
4. Set environment variables

## 🔧 Manual Deployment

### Prerequisites
- Ubuntu 20.04+ (or similar Linux distribution)
- Rust 1.70+
- PostgreSQL 13+
- Nginx (recommended)
- SSL certificate (Let's Encrypt recommended)

### Step-by-Step Manual Deployment

#### 1. System Setup
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install required packages
sudo apt install -y build-essential pkg-config libssl-dev libpq-dev nginx postgresql postgresql-contrib python3-pip

# Install yt-dlp for YouTube video downloads (optional but recommended)
sudo pip3 install yt-dlp

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### 2. Database Setup
```bash
# Start PostgreSQL
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres createdb rocket_blog_prod
sudo -u postgres createuser --superuser rocket_blog_user
sudo -u postgres psql -c "ALTER USER rocket_blog_user PASSWORD 'secure_password_here';"
```

#### 3. Application Setup
```bash
# Clone repository
git clone https://github.com/xiro-codes/rocket_blog.git
cd rocket_blog

# Create production user
sudo useradd -r -s /bin/bash -d /opt/rocket_blog rocket_blog
sudo mkdir -p /opt/rocket_blog
sudo chown rocket_blog:rocket_blog /opt/rocket_blog

# Copy application
sudo cp -r . /opt/rocket_blog/
sudo chown -R rocket_blog:rocket_blog /opt/rocket_blog
```

#### 4. Build Application
```bash
# Switch to application user
sudo -u rocket_blog -s

# Build release version
cd /opt/rocket_blog
just build
# OR: cargo build --release

# Create data directory
mkdir -p /opt/rocket_blog/data
```

#### 5. Configuration
```bash
# Create production config
sudo -u rocket_blog tee /opt/rocket_blog/Rocket.toml > /dev/null << EOF
[default]
address = "127.0.0.1"
port = 8000
limits.file = "1 GB"
limits.data-form = "1 GB" 
limits.form = "1 GB"
data_path = "/opt/rocket_blog/data"

[default.databases.sea_orm]
url = "postgres://rocket_blog_user:secure_password_here@localhost/rocket_blog_prod"

[release]
secret_key = "$(openssl rand -base64 32)"
address = "127.0.0.1"
port = 8000
data_path = "/opt/rocket_blog/data"

[release.databases.sea_orm] 
url = "postgres://rocket_blog_user:secure_password_here@localhost/rocket_blog_prod"
EOF
```

#### 6. Run Migrations
```bash
just migrate
# OR: sudo -u rocket_blog /opt/rocket_blog/target/release/migrations
```

#### 7. Create Systemd Service
```bash
sudo tee /etc/systemd/system/rocket-blog.service > /dev/null << EOF
[Unit]
Description=Rocket Blog Application
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=rocket_blog
Group=rocket_blog
WorkingDirectory=/opt/rocket_blog
Environment=ROCKET_ENV=release
ExecStart=/opt/rocket_blog/target/release/app
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable rocket-blog
sudo systemctl start rocket-blog
sudo systemctl status rocket-blog
```

#### 8. Nginx Configuration
```bash
# Create Nginx config
sudo tee /etc/nginx/sites-available/rocket-blog > /dev/null << EOF
server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://127.0.0.1:8000;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        
        # WebSocket support (for future features)
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # File upload support
        client_max_body_size 1G;
    }
    
    # Static files (optional optimization)
    location /static/ {
        alias /opt/rocket_blog/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
EOF

# Enable site
sudo ln -s /etc/nginx/sites-available/rocket-blog /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

#### 9. SSL Certificate (Let's Encrypt)
```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d your-domain.com

# Test auto-renewal
sudo certbot renew --dry-run
```

## 🔒 Security Configuration

### Firewall Setup
```bash
# Configure UFW
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 'Nginx Full'
sudo ufw enable
```

### Database Security
```bash
# Secure PostgreSQL
sudo -u postgres psql -c "ALTER USER rocket_blog_user CREATEDB;"
sudo -u postgres psql -c "REVOKE ALL ON SCHEMA public FROM public;"
sudo -u postgres psql -c "GRANT ALL ON SCHEMA public TO rocket_blog_user;"
```

### Application Security
```bash
# Set proper file permissions
sudo chmod 600 /opt/rocket_blog/Rocket.toml
sudo chown rocket_blog:rocket_blog /opt/rocket_blog/Rocket.toml

# Secure data directory
sudo chmod 750 /opt/rocket_blog/data
sudo chown -R rocket_blog:rocket_blog /opt/rocket_blog/data
```

## 📊 Monitoring and Maintenance

### Log Management
```bash
# View application logs
sudo journalctl -u rocket-blog -f

# View Nginx logs
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log

# PostgreSQL logs
sudo tail -f /var/log/postgresql/postgresql-13-main.log
```

### Health Checks
```bash
# Check service status
sudo systemctl status rocket-blog
sudo systemctl status postgresql
sudo systemctl status nginx

# Check listening ports
sudo netstat -tlnp | grep :8000
sudo netstat -tlnp | grep :80
sudo netstat -tlnp | grep :443
```

### Backup Strategy
```bash
# Database backup script
sudo tee /opt/rocket_blog/backup.sh > /dev/null << 'EOF'
#!/bin/bash
BACKUP_DIR="/opt/rocket_blog/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Database backup
pg_dump -U rocket_blog_user rocket_blog_prod > $BACKUP_DIR/db_$DATE.sql

# Application data backup
tar -czf $BACKUP_DIR/data_$DATE.tar.gz /opt/rocket_blog/data/

# Keep only last 7 days of backups
find $BACKUP_DIR -name "*.sql" -mtime +7 -delete
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete

echo "Backup completed: $DATE"
EOF

sudo chmod +x /opt/rocket_blog/backup.sh
sudo chown rocket_blog:rocket_blog /opt/rocket_blog/backup.sh

# Add to crontab for daily backups
echo "0 2 * * * /opt/rocket_blog/backup.sh" | sudo -u rocket_blog crontab -
```

### Docker Volume Backup Strategy

For Docker deployments, use the built-in Docker volume backup functionality:

```bash
# Backup Docker volumes (auto-detects environment)
./scripts/docker-deploy.sh backup

# Backup specific environment
./scripts/docker-deploy.sh backup prod   # Production volumes
./scripts/docker-deploy.sh backup dev    # Development volumes

# List available backups
./scripts/docker-deploy.sh backup-list

# Restore from latest backup
./scripts/docker-deploy.sh restore

# Restore specific environment
./scripts/docker-deploy.sh restore prod

# Clean old backups (remove older than 7 days)
./scripts/docker-deploy.sh backup-clean

# Clean old backups (custom retention period)
./scripts/docker-deploy.sh backup-clean 30  # Keep 30 days

# Alternative: Use the backup script directly
./scripts/docker-backup.sh backup           # Auto-detect environment
./scripts/docker-backup.sh backup prod      # Backup production
./scripts/docker-backup.sh restore          # Restore latest
./scripts/docker-backup.sh list             # List backups
./scripts/docker-backup.sh clean 14         # Keep 14 days
```

#### What Gets Backed Up

**Production Environment:**
- `postgres_data` - PostgreSQL database files
- `app_data` - Application uploaded files and data
- `letsencrypt_data` - SSL certificates
- `certbot_webroot` - Certbot validation files
- `nginx_logs` - Nginx access and error logs

**Development Environment:**
- `postgres_data` - PostgreSQL database files
- `app_data` - Application uploaded files and data

#### Backup Location

Backups are stored in `./backups/` directory by default. You can customize this with:

```bash
# Custom backup directory
BACKUP_DIR=/path/to/backups ./scripts/docker-backup.sh backup
```

#### Automated Backups

Add to crontab for automated Docker volume backups:

```bash
# Daily backup at 2 AM
echo "0 2 * * * cd /opt/rocket_blog && ./scripts/docker-deploy.sh backup prod" | crontab -

# Weekly cleanup (keep 30 days)
echo "0 3 * * 0 cd /opt/rocket_blog && ./scripts/docker-deploy.sh backup-clean 30" | crontab -
```

## 🚀 Performance Optimization

### Database Optimization
```sql
-- Connect to PostgreSQL as rocket_blog_user
-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_posts_published ON posts(published);
CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts(created_at);
CREATE INDEX IF NOT EXISTS idx_comments_post_id ON comments(post_id);
CREATE INDEX IF NOT EXISTS idx_post_tags_post_id ON post_tags(post_id);
CREATE INDEX IF NOT EXISTS idx_post_tags_tag_id ON post_tags(tag_id);
```

### Nginx Caching
```nginx
# Add to Nginx config for better performance
server {
    # ... existing config ...
    
    # Enable gzip compression
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;
    
    # Cache static assets
    location ~* \.(jpg|jpeg|png|gif|ico|css|js)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # Cache blog posts for 5 minutes
    location ~* ^/blog/\d+$ {
        proxy_pass http://127.0.0.1:8000;
        proxy_cache_valid 200 5m;
        add_header X-Cache-Status $upstream_cache_status;
    }
}
```

## 🔄 Deployment Updates

### Zero-Downtime Deployment
```bash
# Script for updating application
sudo tee /opt/rocket_blog/deploy.sh > /dev/null << 'EOF'
#!/bin/bash
set -e

echo "Starting deployment..."

# Pull latest code
cd /opt/rocket_blog
git fetch origin
git checkout main
git pull origin main

# Build new version
just build
# OR: cargo build --release

# Run migrations
just migrate
# OR: ./target/release/migrations

# Restart service
sudo systemctl restart rocket-blog

# Wait for service to start
sleep 5

# Check if service is running
if sudo systemctl is-active --quiet rocket-blog; then
    echo "Deployment successful!"
else
    echo "Deployment failed - service not running"
    sudo systemctl status rocket-blog
    exit 1
fi
EOF

sudo chmod +x /opt/rocket_blog/deploy.sh
sudo chown rocket_blog:rocket_blog /opt/rocket_blog/deploy.sh
```

### Database Migrations
```bash
# Before deployment, always backup database
/opt/rocket_blog/backup.sh

# Run migrations
just migrate
# OR: sudo -u rocket_blog /opt/rocket_blog/target/release/migrations

# Verify migration status
just migrate-status
# OR: sudo -u rocket_blog /opt/rocket_blog/target/release/migrations status
```

## 🐛 Troubleshooting

### Common Issues

#### Service Won't Start
```bash
# Check service status
sudo systemctl status rocket-blog

# Check logs
sudo journalctl -u rocket-blog -n 50

# Common fixes:
# 1. Check database connection
# 2. Verify file permissions
# 3. Check port availability
```

#### Database Connection Issues
```bash
# Test database connection
sudo -u rocket_blog psql -U rocket_blog_user -d rocket_blog_prod -h localhost

# Check PostgreSQL status
sudo systemctl status postgresql

# Check PostgreSQL configuration
sudo nano /etc/postgresql/13/main/postgresql.conf
sudo nano /etc/postgresql/13/main/pg_hba.conf
```

#### High Memory Usage
```bash
# Monitor memory usage
htop
sudo systemctl status rocket-blog

# Optimize PostgreSQL
sudo nano /etc/postgresql/13/main/postgresql.conf
# Adjust shared_buffers, work_mem, etc.
```

## 📞 Support

For deployment issues:
1. Check the [Troubleshooting section](#-troubleshooting)
2. Review application logs: `sudo journalctl -u rocket-blog`
3. Create an issue on [GitHub](https://github.com/xiro-codes/rocket_blog/issues)
4. Join our community chat for real-time help

## 🎯 Production Checklist

Before going live:

- [ ] SSL certificate installed and working
- [ ] Firewall configured properly
- [ ] Database secured with strong passwords
- [ ] Backups configured and tested
- [ ] Monitoring and logging set up
- [ ] Domain name configured
- [ ] Email notifications configured
- [ ] Performance optimizations applied
- [ ] Security headers configured in Nginx
- [ ] Regular update procedure established

---

Your Rocket Blog deployment should now be ready for production! 🚀