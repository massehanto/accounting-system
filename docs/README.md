# Indonesian Accounting System

## Architecture Overview

This system follows a microservices architecture with the following components:

### Core Services
- **Auth Service** (Port 3001): User authentication and authorization
- **Company Management** (Port 3002): Company profile and settings
- **Chart of Accounts** (Port 3003): Account structure management

### Business Logic Services
- **General Ledger** (Port 3004): Journal entries and financial transactions
- **Indonesian Tax** (Port 3005): Tax calculations and compliance
- **Accounts Payable** (Port 3006): Vendor management and payments
- **Accounts Receivable** (Port 3007): Customer management and collections
- **Inventory Management** (Port 3008): Stock tracking and valuation

### Supporting Services
- **Reporting Service** (Port 3009): Financial reports and analytics
- **API Gateway** (Port 8080): Request routing and authentication

## Quick Start

1. Configure environment variables in `.env`
2. Set up PostgreSQL databases for each service
3. Run `./scripts/start-services.sh`
4. Access the system at `http://localhost:8080`

## Development

Each service is independently deployable and scalable. Services communicate through HTTP APIs routed via the API Gateway.

### Service Structure