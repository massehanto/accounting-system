#!/bin/bash
# deploy.sh - Streamlined Indonesian Accounting System Deployment Script
# Usage: ./deploy.sh [domain] [environment]

set -euo pipefail

# Configuration
DEPLOY_USER="accounting"
DEPLOY_DIR="/opt/accounting-system"
BACKUP_DIR="/opt/accounting-backups"
LOG_FILE="/var/log/accounting-deploy.log"
POSTGRES_VERSION="16"

# Parse arguments
DOMAIN=${1:-""}
ENVIRONMENT=${2:-"development"}

# Set environment-specific variables
if [[ "$ENVIRONMENT" == "production" || "$ENVIRONMENT" == "prod" ]]; then
    IS_PRODUCTION=true
    echo "🚨 PRODUCTION DEPLOYMENT MODE"
else
    IS_PRODUCTION=false
    echo "🛠️  DEVELOPMENT DEPLOYMENT MODE"
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Service definitions
SERVICES=(
    "auth-service:auth-service"
    "company-management-service:company-management-service"
    "chart-of-accounts-service:chart-of-accounts-service"
    "general-ledger-service:general-ledger-service"
    "indonesian-tax-service:indonesian-tax-service"
    "accounts-payable-service:accounts-payable-service"
    "accounts-receivable-service:accounts-receivable-service"
    "inventory-management-service:inventory-management-service"
    "reporting-service:reporting-service"
    "api-gateway:api-gateway"
)

DATABASES=(
    "auth_service"
    "company_management"
    "chart_of_accounts"
    "general_ledger"
    "indonesian_tax"
    "accounts_payable"
    "accounts_receivable"
    "inventory_management"
)

log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')] $1${NC}" | tee -a "$LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}" | tee -a "$LOG_FILE"
    exit 1
}

check_prerequisites() {
    log "🔍 Checking prerequisites..."
    
    # Check if running as non-root
    if [[ $EUID -eq 0 ]]; then
        error "This script should not be run as root for security reasons"
    fi
    
    # Check required files exist
    local required_files=(
        ".env"
        "Cargo.toml"
        "migrations"
        "configs/nginx.conf"
        "service/systemd-template.service"
        "scripts/backup.sh"
        "scripts/health-check.sh"
        "configs/logrotate.conf"
    )
    
    for file in "${required_files[@]}"; do
        if [[ ! -e "$file" ]]; then
            error "Required file/directory not found: $file"
        fi
    done
    
    log "✅ Prerequisites check passed"
}

install_dependencies() {
    log "📦 Installing system dependencies..."
    
    # Update system
    sudo apt update && sudo apt upgrade -y
    
    # Install PostgreSQL if not present
    if ! command -v psql &> /dev/null; then
        log "Installing PostgreSQL $POSTGRES_VERSION..."
        sudo sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
        wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -
        sudo apt update
        sudo apt install -y postgresql-$POSTGRES_VERSION postgresql-client-$POSTGRES_VERSION postgresql-contrib-$POSTGRES_VERSION
        sudo systemctl start postgresql
        sudo systemctl enable postgresql
    fi
    
    # Install Rust if not present
    if ! command -v cargo &> /dev/null; then
        log "Installing Rust toolchain..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        cargo install sqlx-cli --no-default-features --features postgres
    fi
    
    # Install other dependencies
    sudo apt install -y \
        build-essential pkg-config libssl-dev nginx \
        ufw fail2ban logrotate htop tmux git unzip curl wget \
        netstat-nat jq
    
    log "✅ Dependencies installed successfully"
}

setup_application_structure() {
    log "👤 Setting up application structure..."
    
    # Create application user if it doesn't exist
    if ! id "$DEPLOY_USER" &>/dev/null; then
        sudo useradd -r -s /bin/bash -d "$DEPLOY_DIR" "$DEPLOY_USER"
        log "Created user: $DEPLOY_USER"
    fi
    
    # Create directory structure
    sudo mkdir -p "$DEPLOY_DIR"/{bin,logs,config,frontend,source}
    sudo mkdir -p "$BACKUP_DIR"
    
    # Set ownership and permissions
    sudo chown -R "$DEPLOY_USER:$DEPLOY_USER" "$DEPLOY_DIR"
    sudo chmod -R 755 "$DEPLOY_DIR"
    sudo chmod 750 "$DEPLOY_DIR/config"  # More restrictive for config
    
    log "✅ Application structure created"
}

setup_databases() {
    log "🗄️ Setting up databases..."
    
    # Create database user if it doesn't exist
    if ! sudo -u postgres psql -c "\du" | grep -q "$DEPLOY_USER"; then
        sudo -u postgres createuser --createdb --no-superuser --no-createrole "$DEPLOY_USER"
        log "Created database user: $DEPLOY_USER"
    fi
    
    # Create databases and apply schemas
    for db in "${DATABASES[@]}"; do
        log "Setting up database: $db"
        
        # Create database if it doesn't exist
        if ! sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw "$db"; then
            sudo -u postgres createdb -O "$DEPLOY_USER" "$db"
            log "Created database: $db"
        fi
        
        # Apply schema if exists
        local schema_file="migrations/${db}_schema.sql"
        if [[ -f "$schema_file" ]]; then
            log "Applying schema for $db..."
            sudo -u postgres psql -d "$db" -f "$schema_file" || warn "Failed to apply schema for $db"
        fi
    done
    
    log "✅ Databases setup completed"
}

configure_environment() {
    log "📝 Configuring environment..."
    
    # Copy and modify environment file
    sudo cp .env "$DEPLOY_DIR/config/.env"
    
    # Update environment-specific settings
    if [[ "$IS_PRODUCTION" == true ]]; then
        log "Applying production configurations..."
        
        # Update service host for production
        sudo sed -i 's/SERVICE_HOST=127.0.0.1/SERVICE_HOST=0.0.0.0/' "$DEPLOY_DIR/config/.env"
        
        # Update database connection limits
        sudo sed -i 's/DB_MAX_CONNECTIONS=20/DB_MAX_CONNECTIONS=50/' "$DEPLOY_DIR/config/.env"
        sudo sed -i 's/DB_MIN_CONNECTIONS=5/DB_MIN_CONNECTIONS=10/' "$DEPLOY_DIR/config/.env"
        
        # Update logging level
        sudo sed -i 's/RUST_LOG=info/RUST_LOG=info/' "$DEPLOY_DIR/config/.env"
        
        # Update CORS origins if domain is provided
        if [[ -n "$DOMAIN" ]]; then
            sudo sed -i "s|CORS_ALLOWED_ORIGINS=.*|CORS_ALLOWED_ORIGINS=https://$DOMAIN,https://www.$DOMAIN|" "$DEPLOY_DIR/config/.env"
        fi
        
        # Generate secure JWT secret
        local jwt_secret=$(openssl rand -base64 48)
        sudo sed -i "s/JWT_SECRET=.*/JWT_SECRET=$jwt_secret/" "$DEPLOY_DIR/config/.env"
        
        warn "🔒 Remember to update database passwords in production!"
        warn "🔒 JWT secret has been automatically generated"
    fi
    
    # Set proper permissions
    sudo chmod 600 "$DEPLOY_DIR/config/.env"
    sudo chown "$DEPLOY_USER:$DEPLOY_USER" "$DEPLOY_DIR/config/.env"
    
    log "✅ Environment configured"
}

build_application() {
    log "🔨 Building application..."
    
    # Copy source code
    sudo cp -r . "$DEPLOY_DIR/source/"
    sudo chown -R "$DEPLOY_USER:$DEPLOY_USER" "$DEPLOY_DIR/source/"
    
    cd "$DEPLOY_DIR/source"
    
    # Build backend services
    log "Building Rust services..."
    sudo -u "$DEPLOY_USER" bash -c "source ~/.cargo/env && cargo build --release --workspace --exclude frontend"
    
    # Copy binaries
    sudo mkdir -p "$DEPLOY_DIR/bin"
    for service_info in "${SERVICES[@]}"; do
        IFS=':' read -r service_name binary_name <<< "$service_info"
        if [[ -f "target/release/$binary_name" ]]; then
            sudo cp "target/release/$binary_name" "$DEPLOY_DIR/bin/"
            sudo chmod +x "$DEPLOY_DIR/bin/$binary_name"
            log "✅ Binary copied: $binary_name"
        else
            warn "Binary not found: $binary_name"
        fi
    done
    
    # Build frontend if exists
    if [[ -d "frontend" ]]; then
        log "Building frontend..."
        cd frontend
        if sudo -u "$DEPLOY_USER" bash -c "source ~/.cargo/env && trunk build --release"; then
            sudo cp -r dist/* "$DEPLOY_DIR/frontend/"
            log "✅ Frontend built successfully"
        else
            warn "Frontend build failed"
        fi
        cd ..
    fi
    
    sudo chown -R "$DEPLOY_USER:$DEPLOY_USER" "$DEPLOY_DIR"
    
    log "✅ Application built successfully"
}

create_systemd_services() {
    log "🔧 Creating systemd services..."
    
    for service_info in "${SERVICES[@]}"; do
        IFS=':' read -r service_name binary_name <<< "$service_info"
        
        # Use template and replace placeholders
        sudo cp deployment/systemd-template.service "/etc/systemd/system/accounting-$service_name.service"
        sudo sed -i "s/SERVICE_NAME/$service_name/g" "/etc/systemd/system/accounting-$service_name.service"
        sudo sed -i "s/SERVICE_BINARY/$binary_name/g" "/etc/systemd/system/accounting-$service_name.service"
        
        log "Created systemd service: accounting-$service_name"
    done
    
    sudo systemctl daemon-reload
    log "✅ Systemd services created"
}

configure_nginx() {
    log "🌐 Configuring Nginx..."
    
    # Copy nginx configuration
    sudo cp configs/nginx.conf /etc/nginx/sites-available/app.ae-systems.id
    
    # Update domain if provided
    if [[ -n "$DOMAIN" ]]; then
        sudo sed -i "s/server_name _;/server_name $DOMAIN www.$DOMAIN;/" /etc/nginx/sites-available/app.ae-systems.id
    fi
    
    # Remove default site and enable our site
    sudo rm -f /etc/nginx/sites-enabled/default
    sudo ln -sf /etc/nginx/sites-available/ae-systems.id /etc/nginx/sites-enabled/
    
    # Test configuration
    if sudo nginx -t; then
        sudo systemctl restart nginx
        sudo systemctl enable nginx
        log "✅ Nginx configured and restarted"
    else
        error "Nginx configuration test failed"
    fi
}

setup_logging() {
    log "📋 Setting up logging..."
    
    # Copy logrotate configuration
    sudo cp deployment/logrotate.conf /etc/logrotate.d/accounting
    
    # Create log directories
    sudo mkdir -p /var/log/nginx
    sudo mkdir -p "$DEPLOY_DIR/logs"
    sudo chown "$DEPLOY_USER:$DEPLOY_USER" "$DEPLOY_DIR/logs"
    
    # Test logrotate configuration
    sudo logrotate -d /etc/logrotate.d/accounting
    
    log "✅ Logging configured"
}

configure_security() {
    log "🔒 Configuring security..."
    
    # Configure firewall
    sudo ufw --force reset
    sudo ufw default deny incoming
    sudo ufw default allow outgoing
    sudo ufw allow 22/tcp comment "SSH"
    sudo ufw allow 80/tcp comment "HTTP"
    sudo ufw allow 443/tcp comment "HTTPS"
    
    if [[ "$IS_PRODUCTION" == false ]]; then
        # Allow direct service access in development
        sudo ufw allow 8080/tcp comment "API Gateway Dev"
    fi
    
    sudo ufw --force enable
    
    # Configure fail2ban
    sudo systemctl enable fail2ban
    sudo systemctl start fail2ban
    
    log "✅ Security configured"
}

install_management_tools() {
    log "🛠️ Installing management tools..."
    
    # Install health check script
    sudo cp deployment/health-check.sh /usr/local/bin/accounting-status
    sudo chmod +x /usr/local/bin/accounting-status
    
    # Install backup script
    sudo cp deployment/backup.sh /usr/local/bin/accounting-backup
    sudo chmod +x /usr/local/bin/accounting-backup
    
    # Create wrapper script for easy service management
    sudo tee /usr/local/bin/accounting-manage > /dev/null <<'EOF'
#!/bin/bash
# accounting-manage - Service management wrapper

case "$1" in
    start)
        echo "Starting all accounting services..."
        for service in $(systemctl list-units 'accounting-*' --no-legend | awk '{print $1}'); do
            sudo systemctl start "$service"
        done
        ;;
    stop)
        echo "Stopping all accounting services..."
        for service in $(systemctl list-units 'accounting-*' --no-legend | awk '{print $1}'); do
            sudo systemctl stop "$service"
        done
        ;;
    restart)
        echo "Restarting all accounting services..."
        for service in $(systemctl list-units 'accounting-*' --no-legend | awk '{print $1}'); do
            sudo systemctl restart "$service"
        done
        ;;
    status)
        /usr/local/bin/accounting-status
        ;;
    logs)
        if [[ -n "$2" ]]; then
            sudo journalctl -u "accounting-$2" -f
        else
            echo "Usage: accounting-manage logs <service-name>"
            echo "Available services:"
            systemctl list-units 'accounting-*' --no-legend | awk '{print "  " $1}' | sed 's/accounting-//' | sed 's/.service//'
        fi
        ;;
    backup)
        /usr/local/bin/accounting-backup
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs <service>|backup}"
        exit 1
        ;;
esac
EOF
    
    sudo chmod +x /usr/local/bin/accounting-manage
    
    # Schedule automatic backups
    if ! sudo crontab -l 2>/dev/null | grep -q "accounting-backup"; then
        (sudo crontab -l 2>/dev/null; echo "0 2 * * * /usr/local/bin/accounting-backup") | sudo crontab -
        log "Scheduled daily backups at 2 AM"
    fi
    
    log "✅ Management tools installed"
}

start_services() {
    log "🚀 Starting services..."
    
    # Start services in dependency order
    local start_order=(
        "auth-service"
        "company-management-service"
        "chart-of-accounts-service"
        "general-ledger-service"
        "indonesian-tax-service"
        "accounts-payable-service"
        "accounts-receivable-service"
        "inventory-management-service"
        "reporting-service"
        "api-gateway"
    )
    
    for service_name in "${start_order[@]}"; do
        log "Starting accounting-$service_name..."
        sudo systemctl enable "accounting-$service_name"
        sudo systemctl start "accounting-$service_name"
        
        # Wait and verify
        sleep 3
        if systemctl is-active --quiet "accounting-$service_name"; then
            log "✅ $service_name started successfully"
        else
            warn "⚠️ $service_name may have issues, check logs with: sudo journalctl -u accounting-$service_name"
        fi
    done
    
    log "✅ All services started"
}

setup_ssl() {
    if [[ -n "$DOMAIN" && "$IS_PRODUCTION" == true ]]; then
        log "🔐 Setting up SSL for $DOMAIN..."
        
        # Install certbot
        sudo apt install -y certbot python3-certbot-nginx
        
        # Obtain SSL certificate
        if sudo certbot --nginx -d "$DOMAIN" --non-interactive --agree-tos --email "admin@$DOMAIN" --redirect; then
            log "✅ SSL certificate installed for $DOMAIN"
        else
            warn "SSL setup failed. Run manually: sudo certbot --nginx -d $DOMAIN"
        fi
    fi
}

run_health_check() {
    log "🏥 Running post-deployment health check..."
    
    sleep 10  # Allow services to fully start
    
    if /usr/local/bin/accounting-status; then
        log "✅ Health check passed"
    else
        warn "⚠️ Health check found issues - review the output above"
    fi
}

print_summary() {
    log "🎉 Deployment completed successfully!"
    echo ""
    echo "📋 Deployment Summary:"
    echo "   🏢 Environment: $ENVIRONMENT"
    echo "   🌐 Domain: ${DOMAIN:-"localhost"}"
    echo "   📁 Install Directory: $DEPLOY_DIR"
    echo "   🗄️ Databases: ${#DATABASES[@]} created"
    echo "   🔧 Services: ${#SERVICES[@]} installed"
    echo ""
    echo "🛠️ Management Commands:"
    echo "   📊 System Status: accounting-status"
    echo "   🔧 Manage Services: accounting-manage {start|stop|restart|status}"
    echo "   📋 View Logs: accounting-manage logs <service-name>"
    echo "   💾 Manual Backup: accounting-backup"
    echo ""
    echo "📂 Important Paths:"
    echo "   📄 Config: $DEPLOY_DIR/config/.env"
    echo "   📋 Logs: $DEPLOY_DIR/logs/ and /var/log/nginx/"
    echo "   💾 Backups: $BACKUP_DIR"
    echo ""
    echo "🌐 Access Points:"
    if [[ -n "$DOMAIN" ]]; then
        echo "   🔗 Application: https://$DOMAIN"
        echo "   📊 Monitoring: https://$DOMAIN/monitoring/"
    else
        echo "   🔗 Application: http://localhost:8080"
        echo "   📊 Monitoring: http://localhost:8080/monitoring/"
    fi
    echo "   🏥 Health Check: /health"
    echo ""
    echo "🔧 Next Steps:"
    echo "   1. Review configuration: $DEPLOY_DIR/config/.env"
    echo "   2. Create initial admin user via API"
    echo "   3. Test all functionality"
    if [[ "$IS_PRODUCTION" == true ]]; then
        echo "   4. Update database passwords for production"
        echo "   5. Configure monitoring and alerting"
        echo "   6. Set up regular backup verification"
    fi
    echo ""
}

main() {
    log "🚀 Starting Indonesian Accounting System deployment..."
    log "Target: $ENVIRONMENT environment${DOMAIN:+ for $DOMAIN}"
    
    # Create log file
    sudo touch "$LOG_FILE"
    sudo chown "$USER:$USER" "$LOG_FILE"
    
    # Run deployment steps
    check_prerequisites
    install_dependencies
    setup_application_structure
    setup_databases
    configure_environment
    build_application
    create_systemd_services
    configure_nginx
    setup_logging
    configure_security
    install_management_tools
    start_services
    setup_ssl
    run_health_check
    print_summary
}

# Run main function with all arguments
main "$@"