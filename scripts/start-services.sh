#!/bin/bash

# Start all services for Indonesian Accounting System

echo "Starting Indonesian Accounting System Services..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to start a service
start_service() {
    local service_name=$1
    local service_path=$2
    local port=$3
    
    echo -e "${YELLOW}Starting $service_name on port $port...${NC}"
    
    cd "$service_path" || {
        echo -e "${RED}Failed to change to $service_path directory${NC}"
        return 1
    }
    
    cargo run --release > "../logs/${service_name}.log" 2>&1 &
    local pid=$!
    echo $pid > "../pids/${service_name}.pid"
    
    echo -e "${GREEN}Started $service_name (PID: $pid)${NC}"
    cd - > /dev/null
}

# Create directories for logs and PIDs
mkdir -p logs pids

# Start services in order
echo "Starting core services first..."

start_service "auth-service" "services/auth" "3001"
sleep 2

start_service "company-management-service" "services/company-management" "3002"
sleep 2

start_service "chart-of-accounts-service" "services/chart-of-accounts" "3003"
sleep 2

start_service "general-ledger-service" "services/general-ledger" "3004"
sleep 2

echo "Starting business services..."

start_service "indonesian-tax-service" "services/indonesian-tax" "3005"
sleep 2

start_service "accounts-payable-service" "services/accounts-payable" "3006"
sleep 2

start_service "accounts-receivable-service" "services/accounts-receivable" "3007"
sleep 2

start_service "inventory-management-service" "services/inventory-management" "3008"
sleep 2

start_service "reporting-service" "services/reporting" "3009"
sleep 2

echo "Starting API Gateway..."
start_service "api-gateway" "services/api-gateway" "3000"

echo -e "${GREEN}All services started successfully!${NC}"
echo -e "${YELLOW}API Gateway is available at: http://localhost:3000${NC}"
echo ""
echo "Service status:"
echo "  - API Gateway:              http://localhost:3000"
echo "  - Auth Service:             http://localhost:3001"
echo "  - Company Management:       http://localhost:3002"
echo "  - Chart of Accounts:        http://localhost:3003"
echo "  - General Ledger:           http://localhost:3004"
echo "  - Indonesian Tax:           http://localhost:3005"
echo "  - Accounts Payable:         http://localhost:3006"
echo "  - Accounts Receivable:      http://localhost:3007"
echo "  - Inventory Management:     http://localhost:3008"
echo "  - Reporting:                http://localhost:3009"
echo ""
echo "To stop services, run: ./scripts/stop-services.sh"
echo "To check logs, run: tail -f logs/[service-name].log"