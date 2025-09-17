#!/bin/bash

# Check health of all services

echo "Checking Indonesian Accounting System Services Health..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check service health
check_service() {
    local service_name=$1
    local port=$2
    local endpoint="http://localhost:${port}/health"
    
    printf "%-30s " "$service_name:"
    
    if curl -s --connect-timeout 5 "$endpoint" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ HEALTHY${NC}"
        return 0
    else
        echo -e "${RED}✗ UNHEALTHY${NC}"
        return 1
    fi
}

echo "Service Health Status:"
echo "=================================="

# Check all services
healthy=0
total=0

check_service "API Gateway" "3000" && ((healthy++))
((total++))

check_service "Auth Service" "3001" && ((healthy++))
((total++))

check_service "Company Management" "3002" && ((healthy++))
((total++))

check_service "Chart of Accounts" "3003" && ((healthy++))
((total++))

check_service "General Ledger" "3004" && ((healthy++))
((total++))

check_service "Indonesian Tax" "3005" && ((healthy++))
((total++))

check_service "Accounts Payable" "3006" && ((healthy++))
((total++))

check_service "Accounts Receivable" "3007" && ((healthy++))
((total++))

check_service "Inventory Management" "3008" && ((healthy++))
((total++))

check_service "Reporting" "3009" && ((healthy++))
((total++))

echo "=================================="
echo -e "Overall Status: ${healthy}/${total} services healthy"

if [ $healthy -eq $total ]; then
    echo -e "${GREEN}All services are running properly!${NC}"
    exit 0
else
    echo -e "${RED}Some services are not responding.${NC}"
    echo "Check logs in the 'logs' directory for more information."
    exit 1
fi