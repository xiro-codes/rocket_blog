#!/bin/bash

# Docker Volume Backup Script for Rocket Blog
# Backs up Docker volumes to filesystem for backup and recovery

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BACKUP_DIR="${BACKUP_DIR:-$PROJECT_DIR/backups}"
DATE=$(date +%Y%m%d_%H%M%S)

# Docker compose file paths
PROD_COMPOSE_FILE="$SCRIPT_DIR/docker/docker-compose.yml"
DEV_COMPOSE_FILE="$SCRIPT_DIR/docker/docker-compose.dev.yml"

function show_help() {
    echo "Docker Volume Backup Script for Rocket Blog"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  backup [env]        Backup all volumes (env: prod|dev, default: auto-detect)"
    echo "  restore [env]       Restore volumes from latest backup"
    echo "  restore-from [file] Restore from specific backup file"
    echo "  list                List available backups"
    echo "  clean [days]        Remove backups older than N days (default: 7)"
    echo "  help                Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  BACKUP_DIR          Backup directory (default: ./backups)"
    echo ""
    echo "Examples:"
    echo "  $0 backup           # Auto-detect environment and backup"
    echo "  $0 backup prod      # Backup production volumes"
    echo "  $0 backup dev       # Backup development volumes"
    echo "  $0 restore          # Restore from latest backup"
    echo "  $0 restore-from backup_20241201_120000.tar.gz"
    echo "  $0 clean 30         # Remove backups older than 30 days"
}

function detect_environment() {
    # Check which containers are running to determine environment
    if docker compose -f "$PROD_COMPOSE_FILE" ps --services --filter "status=running" | grep -q "nginx"; then
        echo "prod"
    elif docker compose -f "$DEV_COMPOSE_FILE" ps --services --filter "status=running" | grep -q "app"; then
        echo "dev"
    else
        echo "none"
    fi
}

function get_compose_file() {
    local env="$1"
    case "$env" in
        prod) echo "$PROD_COMPOSE_FILE" ;;
        dev) echo "$DEV_COMPOSE_FILE" ;;
        *) echo "" ;;
    esac
}

function get_volume_prefix() {
    local env="$1"
    case "$env" in
        prod) echo "rocket_blog" ;;
        dev) echo "docker" ;;
        *) echo "rocket_blog" ;;
    esac
}

function backup_volumes() {
    local env="${1:-$(detect_environment)}"
    
    if [ "$env" = "none" ]; then
        echo "No running Docker containers detected. Please specify environment: prod or dev"
        exit 1
    fi
    
    local compose_file=$(get_compose_file "$env")
    local volume_prefix=$(get_volume_prefix "$env")
    
    echo "Backing up Docker volumes for environment: $env"
    echo "Using compose file: $compose_file"
    
    # Create backup directory
    mkdir -p "$BACKUP_DIR"
    
    # Create temporary directory for this backup
    local temp_dir=$(mktemp -d)
    local backup_base_name="docker_volumes_${env}_${DATE}"
    local backup_temp_dir="$temp_dir/$backup_base_name"
    mkdir -p "$backup_temp_dir"
    
    echo "Temporary backup directory: $backup_temp_dir"
    
    # Backup each volume
    local volumes=()
    case "$env" in
        prod)
            volumes=("postgres_data" "app_data" "letsencrypt_data" "certbot_webroot" "nginx_logs")
            ;;
        dev)
            volumes=("postgres_data" "app_data")
            ;;
    esac
    
    for volume in "${volumes[@]}"; do
        local full_volume_name="${volume_prefix}_${volume}"
        echo "Backing up volume: $full_volume_name"
        
        # Check if volume exists
        if ! docker volume inspect "$full_volume_name" >/dev/null 2>&1; then
            echo "Warning: Volume $full_volume_name not found, skipping..."
            continue
        fi
        
        # Create volume backup using a temporary container
        docker run --rm \
            -v "$full_volume_name:/source:ro" \
            -v "$backup_temp_dir:/backup" \
            alpine:latest \
            tar -czf "/backup/${volume}.tar.gz" -C /source .
        
        echo "  ✓ Backed up $volume ($(du -h "$backup_temp_dir/${volume}.tar.gz" | cut -f1))"
    done
    
    # Create metadata file
    cat > "$backup_temp_dir/backup_info.txt" << EOF
Rocket Blog Docker Volume Backup
Generated: $(date)
Environment: $env
Hostname: $(hostname)
Docker Version: $(docker --version)
Volumes Backed Up: $(IFS=, ; echo "${volumes[*]}")
EOF
    
    # Create final backup archive
    local final_backup="$BACKUP_DIR/${backup_base_name}.tar.gz"
    cd "$temp_dir"
    tar -czf "$final_backup" "$backup_base_name"
    
    # Cleanup temporary directory
    rm -rf "$temp_dir"
    
    echo ""
    echo "✅ Backup completed successfully!"
    echo "Backup file: $final_backup"
    echo "Size: $(du -h "$final_backup" | cut -f1)"
    
    # Database-specific backup (using pg_dump for better consistency)
    if [ "$env" = "prod" ] || [ "$env" = "dev" ]; then
        local db_backup="$BACKUP_DIR/database_${env}_${DATE}.sql"
        echo ""
        echo "Creating database dump for additional redundancy..."
        
        if docker compose -f "$compose_file" exec -T postgres pg_dump -U master -d tdavis_dev > "$db_backup" 2>/dev/null; then
            echo "✅ Database dump created: $db_backup ($(du -h "$db_backup" | cut -f1))"
        else
            echo "⚠️  Database dump failed (container may not be running)"
            rm -f "$db_backup"
        fi
    fi
}

function list_backups() {
    echo "Available backups in $BACKUP_DIR:"
    echo ""
    
    if [ ! -d "$BACKUP_DIR" ] || [ -z "$(ls -A "$BACKUP_DIR" 2>/dev/null)" ]; then
        echo "No backups found."
        return
    fi
    
    echo "Docker Volume Backups:"
    ls -lh "$BACKUP_DIR"/docker_volumes_*.tar.gz 2>/dev/null | while read -r line; do
        echo "  $line"
    done
    
    echo ""
    echo "Database Dumps:"
    ls -lh "$BACKUP_DIR"/database_*.sql 2>/dev/null | while read -r line; do
        echo "  $line"
    done
}

function restore_volumes() {
    local env="${1:-$(detect_environment)}"
    local backup_file="$2"
    
    if [ -z "$backup_file" ]; then
        # Find latest backup for environment
        backup_file=$(ls -t "$BACKUP_DIR"/docker_volumes_${env}_*.tar.gz 2>/dev/null | head -1)
        if [ -z "$backup_file" ]; then
            echo "No backups found for environment: $env"
            exit 1
        fi
    fi
    
    if [ ! -f "$backup_file" ]; then
        echo "Backup file not found: $backup_file"
        exit 1
    fi
    
    echo "Restoring Docker volumes from: $backup_file"
    echo "Environment: $env"
    echo ""
    echo "⚠️  WARNING: This will overwrite existing volume data!"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Restore cancelled."
        exit 0
    fi
    
    local compose_file=$(get_compose_file "$env")
    local volume_prefix=$(get_volume_prefix "$env")
    
    # Stop services to ensure data consistency
    echo "Stopping services..."
    docker compose -f "$compose_file" down
    
    # Create temporary directory for restore
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Extract backup
    echo "Extracting backup..."
    tar -xzf "$backup_file"
    
    local backup_dir=$(ls -d docker_volumes_*/ | head -1)
    if [ -z "$backup_dir" ]; then
        echo "Invalid backup file format"
        rm -rf "$temp_dir"
        exit 1
    fi
    
    # Restore each volume
    for volume_file in "$backup_dir"/*.tar.gz; do
        if [ ! -f "$volume_file" ]; then
            continue
        fi
        
        local volume_name=$(basename "$volume_file" .tar.gz)
        local full_volume_name="${volume_prefix}_${volume_name}"
        
        echo "Restoring volume: $full_volume_name"
        
        # Remove existing volume
        docker volume rm "$full_volume_name" 2>/dev/null || true
        
        # Create new volume
        docker volume create "$full_volume_name"
        
        # Restore data
        docker run --rm \
            -v "$full_volume_name:/target" \
            -v "$temp_dir/$backup_dir:/backup:ro" \
            alpine:latest \
            tar -xzf "/backup/${volume_name}.tar.gz" -C /target
        
        echo "  ✓ Restored $volume_name"
    done
    
    # Cleanup
    rm -rf "$temp_dir"
    
    echo ""
    echo "✅ Volume restore completed!"
    echo "You can now start your services with:"
    echo "  ./scripts/docker-deploy.sh $env"
}

function clean_old_backups() {
    local days="${1:-7}"
    
    echo "Removing backups older than $days days from $BACKUP_DIR"
    
    if [ ! -d "$BACKUP_DIR" ]; then
        echo "Backup directory does not exist: $BACKUP_DIR"
        return
    fi
    
    local count=0
    
    # Clean volume backups
    while IFS= read -r -d '' file; do
        rm -f "$file"
        echo "Removed: $(basename "$file")"
        ((count++))
    done < <(find "$BACKUP_DIR" -name "docker_volumes_*.tar.gz" -mtime +"$days" -print0 2>/dev/null)
    
    # Clean database dumps
    while IFS= read -r -d '' file; do
        rm -f "$file"
        echo "Removed: $(basename "$file")"
        ((count++))
    done < <(find "$BACKUP_DIR" -name "database_*.sql" -mtime +"$days" -print0 2>/dev/null)
    
    echo "Removed $count old backup files."
}

# Main command handling
case "${1:-help}" in
    backup)
        backup_volumes "$2"
        ;;
    restore)
        restore_volumes "$2"
        ;;
    restore-from)
        if [ -z "$2" ]; then
            echo "Error: Backup file path required"
            show_help
            exit 1
        fi
        # Extract environment from filename or use auto-detect
        env=$(echo "$2" | grep -o '_\(prod\|dev\)_' | tr -d '_' || echo "")
        restore_volumes "$env" "$2"
        ;;
    list)
        list_backups
        ;;
    clean)
        clean_old_backups "$2"
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac