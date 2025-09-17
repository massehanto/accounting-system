#!/bin/bash

# Stop all services for Indonesian Accounting System

echo "Stopping Indonesian Accounting System Services..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to stop a service
stop_service() {
    local service_name=$1
    local pid_file="pids/${service_name}.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        echo -e "${YELLOW}Stopping $service_name (PID: $pid)...${NC}"
        
        if kill -TERM "$pid" 2>/dev/null; then
            # Wait for graceful shutdown
            sleep 2
            
            # Check if process is still running
            if kill -0 "$pid" 2>/dev/null; then
                echo -e "${RED}Service didn't stop gracefully, forcing shutdown...${NC}"
                kill -KILL "$pid" 2>/dev/null
            fi
            
            echo -e "${GREEN}Stopped $service_name${NC}"
        else
            echo -e "${RED}Failed to stop $service_name (PID: $pid)${NC}"
        fi
        
        rm -f "$pid_file"
    else
        echo -e "${YELLOW}No PID file found for $service_name${NC}"
    fi
}

# List of all services
services=(
    "api-gateway"
    "reporting-service"
    "inventory-management-service"
    "accounts-receivable-service"
    "accounts-payable-service"
    "indonesian-tax-service"
    "general-ledger-service"
    "chart-of-accounts-service"
    "company-management-service"
    "auth-service"
)

# Stop services in reverse order
for service in "${services[@]}"; do
    stop_service "$service"
done

# Clean up any remaining processes
echo -e "${YELLOW}Cleaning up any remaining processes...${NC}"
pkill -f "auth-service\|company-management-service\|chart-of-accounts-service\|general-ledger-service\|indonesian-tax-service\|accounts-payable-service\|accounts-receivable-service\|inventory-management-service\|reporting-service\|api-gateway" 2>/dev/null

# Remove PID directory if empty
if [ -d "pids" ] && [ -z "$(ls -A pids)" ]; then
    rmdir pids
fi

echo -e "${GREEN}All services stopped successfully!${NC}"