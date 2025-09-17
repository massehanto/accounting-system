#!/bin/bash

echo "Stopping Indonesian Accounting System services..."

# Kill all related processes
pkill -f "auth-service"
pkill -f "company-management-service"
pkill -f "chart-of-accounts-service"
pkill -f "general-ledger-service"
pkill -f "indonesian-tax-service"
pkill -f "accounts-payable-service"
pkill -f "accounts-receivable-service"
pkill -f "inventory-management-service"
pkill -f "reporting-service"
pkill -f "api-gateway"

echo "All services stopped!"