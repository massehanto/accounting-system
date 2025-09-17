#!/bin/bash

# Database setup script for Indonesian Accounting System

DB_HOST=${DB_HOST:-localhost}
DB_PORT=${DB_PORT:-5432}
DB_USER=${DB_USER:-accounting_db_user}

# List of databases to create
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

echo "Setting up databases for Indonesian Accounting System..."

# Create databases
for db in "${DATABASES[@]}"; do
    echo "Creating database: $db"
    createdb -h $DB_HOST -p $DB_PORT -U $DB_USER $db 2>/dev/null || echo "Database $db already exists"
done

echo "Database setup completed!"
echo "Don't forget to:"
echo "1. Set appropriate passwords in .env file"
echo "2. Grant necessary permissions to the database user"
echo "3. Run migrations for each service"