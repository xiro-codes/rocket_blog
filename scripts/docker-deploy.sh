#!/bin/bash

# Helper script for managing the Rocket Blog Docker deployment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

function show_help() {
    echo "Rocket Blog Docker Management Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  dev                 Start development environment (no SSL)"
    echo "  dev-live            Start development with live template/static reloading"
    echo "  prod                Start production environment (with SSL)"
    echo "  setup-ssl          Generate initial SSL certificates"
    echo "  renew-ssl          Force SSL certificate renewal"
    echo "  status             Show service status"
    echo "  logs [service]     Show logs (optional service name)"
    echo "  backup [env]       Backup Docker volumes (env: prod|dev, auto-detect if not specified)"
    echo "  restore [env]      Restore Docker volumes from latest backup"
    echo "  backup-list        List available backups"
    echo "  backup-clean [days] Remove backups older than N days (default: 7)"
    echo "  stop               Stop all services"
    echo "  clean              Stop and remove all containers/volumes"
    echo "  help               Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 dev              # Start development environment"
    echo "  $0 dev-live         # Start development with live template reloading"
    echo "  $0 prod             # Start production with SSL"
    echo "  $0 logs nginx       # Show nginx logs only"
    echo "  $0 setup-ssl       # Generate SSL certificates"
    echo "  $0 backup           # Backup volumes (auto-detect environment)"
    echo "  $0 backup prod      # Backup production volumes"
    echo "  $0 restore dev      # Restore development volumes"
    echo "  $0 backup-clean 30  # Remove backups older than 30 days"
}

function start_dev() {
    echo "Starting development environment..."
    docker compose -f scripts/docker/docker-compose.dev.yml up -d --build
    echo ""
    echo "Development environment started!"
    echo "  • App: http://localhost:8000"
    echo "  • pgAdmin: http://localhost:5050"
    echo "  • Database: localhost:5432"
    echo ""
    echo "This uses production builds compiled in a clean container environment."
    echo "For live template/static file editing, use: $0 dev-live"
}

function start_dev_live() {
    echo "Starting development environment with live template reloading..."
    docker compose -f scripts/docker/docker-compose.dev.live.yml up -d --build
    echo ""
    echo "Live development environment started!"
    echo "  • App: http://localhost:8000 (templates/static files auto-reload)"
    echo "  • pgAdmin: http://localhost:5050"
    echo "  • Database: localhost:5432"
    echo ""
    echo "Your templates and static files are mounted into the container."
    echo "Changes to HTML templates and CSS/JS will be immediately visible."
    echo "View logs with: $0 logs app"
}

function start_prod() {
    if [ ! -f "/var/lib/docker/volumes/rocket_blog_letsencrypt_data/_data/live/blog.tdavis.dev/fullchain.pem" ]; then
        echo "SSL certificates not found. Running setup first..."
        setup_ssl
    fi
    
    echo "Starting production environment..."
    docker compose -f scripts/docker/docker-compose.yml up -d --build
    echo ""
    echo "Production environment started!"
    echo "  • App: https://blog.tdavis.dev"
    echo "  • pgAdmin: http://localhost:5050"
}

function setup_ssl() {
    echo "Setting up SSL certificates..."
		sh ./scripts/setup-ssl.sh
}

function renew_ssl() {
    echo "Renewing SSL certificates..."
    docker compose -f scripts/docker/docker-compose.yml exec nginx certbot renew --force-renewal
    docker compose -f scripts/docker/docker-compose.yml exec nginx nginx -s reload
    echo "SSL certificates renewed and nginx reloaded."
}

function show_status() {
    echo "Service Status:"
    docker compose -f scripts/docker/docker-compose.yml ps
}

function show_logs() {
    if [ -n "$1" ]; then
        echo "Showing logs for service: $1"
        docker compose -f scripts/docker/docker-compose.yml logs -f "$1"
    else
        echo "Showing logs for all services:"
        docker compose -f scripts/docker/docker-compose.yml logs -f
    fi
}

function stop_services() {
    echo "Stopping all services..."
    docker compose -f scripts/docker/docker-compose.yml down
    docker compose -f scripts/docker/docker-compose.dev.yml down 2>/dev/null || true
    docker compose -f scripts/docker/docker-compose.dev.live.yml down 2>/dev/null || true
    echo "All services stopped."
}

function clean_all() {
    echo "WARNING: This will remove all containers, volumes, and data!"
    read -p "Are you sure? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Cleaning up..."
        docker compose -f scripts/docker/docker-compose.yml down -v --remove-orphans
        docker compose -f scripts/docker/docker-compose.dev.yml down -v --remove-orphans 2>/dev/null || true
        docker compose -f scripts/docker/docker-compose.dev.live.yml down -v --remove-orphans 2>/dev/null || true
        echo "Cleanup complete."
    else
        echo "Cleanup cancelled."
    fi
}

function backup_volumes() {
    "$SCRIPT_DIR/docker-backup.sh" backup "$1"
}

function restore_volumes() {
    "$SCRIPT_DIR/docker-backup.sh" restore "$1"
}

function list_backups() {
    "$SCRIPT_DIR/docker-backup.sh" list
}

function clean_backups() {
    "$SCRIPT_DIR/docker-backup.sh" clean "$1"
}

case "${1:-help}" in
    dev)
        start_dev
        ;;
    dev-live)
        start_dev_live
        ;;
    prod)
        start_prod
        ;;
    setup-ssl)
        setup_ssl
        ;;
    renew-ssl)
        renew_ssl
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs "$2"
        ;;
    backup)
        backup_volumes "$2"
        ;;
    restore)
        restore_volumes "$2"
        ;;
    backup-list)
        list_backups
        ;;
    backup-clean)
        clean_backups "$2"
        ;;
    stop)
        stop_services
        ;;
    clean)
        clean_all
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
