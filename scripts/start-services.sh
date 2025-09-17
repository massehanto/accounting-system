#!/bin/bash

# Load environment variables
source .env

# Start services in order
echo "Starting Indonesian Accounting System..."

# Start core services first
echo "Starting Auth Service..."
cargo run --bin auth-service &
AUTH_PID=$!

echo "Starting Company Management Service..."
cargo run --bin company-management-service &
COMPANY_PID=$!

echo "Starting Chart of Accounts Service..."
cargo run --bin chart-of-accounts-service &
COA_PID=$!

# Wait a bit for core services to start
sleep 5

# Start business logic services
echo "Starting General Ledger Service..."
cargo run --bin general-ledger-service &
GL_PID=$!

echo "Starting Indonesian Tax Service..."
cargo run --bin indonesian-tax-service &
TAX_PID=$!

echo "Starting Accounts Payable Service..."
cargo run --bin accounts-payable-service &
AP_PID=$!

echo "Starting Accounts Receivable Service..."
cargo run --bin accounts-receivable-service &
AR_PID=$!

echo "Starting Inventory Management Service..."
cargo run --bin inventory-management-service &
INV_PID=$!

# Wait for business services
sleep 5

# Start reporting service
echo "Starting Reporting Service..."
cargo run --bin reporting-service &
REP_PID=$!

# Finally start API Gateway
echo "Starting API Gateway..."
cargo run --bin api-gateway &
GATEWAY_PID=$!

echo "All services started!"
echo "API Gateway: http://localhost:8080"

# Wait for any service to exit
wait $AUTH_PID $COMPANY_PID $COA_PID $GL_PID $TAX_PID $AP_PID $AR_PID $INV_PID $REP_PID $GATEWAY_PID