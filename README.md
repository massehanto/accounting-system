# Indonesian Accounting System - Microservices Architecture

A comprehensive accounting system built specifically for Indonesian businesses, featuring compliance with local regulations, tax calculations (PPN, PPh), and NPWP validation.

## Architecture Overview

This system follows microservices architecture principles with the following services:

### Core Services
- **API Gateway** (Port 3000) - Central entry point and request routing
- **Auth Service** (Port 3001) - Authentication and user management  
- **Company Management** (Port 3002) - Company profiles and settings

### Accounting Services
- **Chart of Accounts** (Port 3003) - Account structure management
- **General Ledger** (Port 3004) - Journal entries and posting
- **Accounts Payable** (Port 3006) - Vendor and invoice management
- **Accounts Receivable** (Port 3007) - Customer and billing management
- **Inventory Management** (Port 3008) - Stock tracking and valuation

### Specialized Services
- **Indonesian Tax** (Port 3005) - PPN, PPh calculations and e-Faktur
- **Reporting** (Port 3009) - Financial reports and export capabilities

### Shared Libraries
- **Common** - Shared models, errors, and utilities
- **Database** - Database connections and migrations
- **Auth** - JWT handling and middleware
- **Utils** - Indonesian-specific validations and formatting

## Quick Start

### Prerequisites
- Rust 1.70+
- PostgreSQL 14+
- Git

### Setup

1. **Clone and Setup Database**
   ```bash
   git clone <repository>
   cd indonesian-accounting-system
   
   # Setup databases
   ./scripts/setup-databases.sh
   
   # Run migrations
   ./scripts/run-migrations.sh
   ```

2. **Configure Environment**
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials and JWT secret
   ```

3. **Build All Services**
   ```bash
   ./scripts/build-all.sh
   ```

4. **Start All Services**
   ```bash
   ./scripts/start-services.sh
   ```

5. **Check Service Health**
   ```bash
   ./scripts/check-services.sh
   ```

### API Access

Once all services are running:
- **API Gateway**: http://localhost:3000
- **Health Check**: http://localhost:3000/health

## Indonesian Compliance Features

### Tax Support
- **PPN (VAT)**: 11% calculation and reporting
- **PPh 21**: Employee income tax
- **PPh 22/23**: Withholding taxes
- **E-Faktur**: Electronic invoice integration
- **Tax period reporting**: Monthly and annual

### Business Features
- **NPWP Validation**: Indonesian tax number validation
- **Indonesian Chart of Accounts**: Pre-built templates
- **Rupiah (IDR) Support**: Proper currency formatting
- **Indonesian Date Formats**: DD/MM/YYYY formatting
- **Business License**: NIB validation support

## Development

### Build Individual Service
```bash
cargo build -p auth-service
cargo run -p auth-service
```

### Run Tests
```bash
cargo test --workspace
```

### Add New Service
1. Create service directory: `services/new-service/`
2. Add to workspace `Cargo.toml`
3. Implement service structure
4. Add to API Gateway routing
5. Update scripts

### Database Migrations
```bash
# Add new migration to shared/database/src/migrations.rs
# Then run:
./scripts/run-migrations.sh
```

## Production Deployment

### Without Docker/Kubernetes
1. **Build Release Binaries**
   ```bash
   cargo build --workspace --release
   ```

2. **Deploy Services to Servers**
   ```bash
   # Copy target/release/* binaries to servers
   # Setup systemd services for each
   # Configure reverse proxy (nginx/Apache)
   ```

3. **Database Setup**
   - Setup PostgreSQL cluster
   - Run migrations on production
   - Configure connection pooling

4. **Monitoring**
   - Setup log aggregation
   - Configure health check endpoints
   - Monitor service communication

## API Documentation

### Authentication
```bash
POST /api/v1/auth/login
POST /api/v1/auth/register  
POST /api/v1/auth/refresh
```

### Company Management
```bash
GET    /api/v1/companies
POST   /api/v1/companies
PUT    /api/v1/companies/:id
```

### Financial Operations
```bash
GET    /api/v1/accounts          # Chart of accounts
POST   /api/v1/ledger/entries    # Journal entries
GET    /api/v1/reports/balance-sheet
```

### Indonesian Tax
```bash
POST   /api/v1/tax/ppn/calculate
GET    /api/v1/tax/reports/monthly
```

## Service Directory Structure

```
indonesian-accounting-system/
├── Cargo.toml                     # Workspace configuration
├── .env.example                   # Environment template
├── README.md                      # This file
├── scripts/                       # Utility scripts
│   ├── setup-databases.sh        # Database initialization
│   ├── run-migrations.sh         # Database migrations
│   ├── start-services.sh         # Start all services
│   ├── stop-services.sh          # Stop all services
│   ├── check-services.sh         # Health check
│   └── build-all.sh              # Build and test
├── services/                      # Microservices
│   ├── api-gateway/              # Port 3000
│   ├── auth/                     # Port 3001
│   ├── company-management/       # Port 3002
│   ├── chart-of-accounts/        # Port 3003
│   ├── general-ledger/           # Port 3004
│   ├── indonesian-tax/           # Port 3005
│   ├── accounts-payable/         # Port 3006
│   ├── accounts-receivable/      # Port 3007
│   ├── inventory-management/     # Port 3008
│   └── reporting/                # Port 3009
└── shared/                       # Shared libraries
    ├── common/                   # Common models and utilities
    ├── database/                 # Database connections and migrations
    ├── auth/                     # JWT and authentication middleware
    └── utils/                    # Indonesian-specific utilities
```

## Database Schema

The system uses multiple PostgreSQL databases for service isolation:

- `auth_service` - User authentication and sessions
- `company_management` - Company profiles and settings
- `chart_of_accounts` - Account structure and templates
- `general_ledger` - Journal entries and balances
- `indonesian_tax` - Tax configurations and transactions
- `accounts_payable` - Vendors and payable invoices
- `accounts_receivable` - Customers and receivable invoices
- `inventory_management` - Items and stock transactions

## Configuration

### Environment Variables

```bash
# Database
DATABASE_URL="postgresql://user:password@localhost:5432/accounting_system"

# Security
JWT_SECRET="your-secret-key-here"

# Service Ports
API_GATEWAY_BIND="0.0.0.0:3000"
AUTH_SERVICE_BIND="0.0.0.0:3001"
# ... (other service binds)

# Email (for auth service)
SMTP_HOST="smtp.gmail.com"
SMTP_PORT="587"
SMTP_USERNAME="your-email@gmail.com"
SMTP_PASSWORD="your-app-password"
```

### Indonesian Localization

The system includes Indonesian-specific features:

- **Currency**: Indonesian Rupiah (IDR) formatting
- **Dates**: DD/MM/YYYY format with Indonesian month names
- **Tax System**: PPN, PPh calculations according to Indonesian law
- **Business Numbers**: NPWP and NIB validation
- **Account Names**: Chart of accounts in Bahasa Indonesia

## Testing

### Unit Tests
```bash
# Test all services
cargo test --workspace

# Test specific service
cargo test -p auth-service
```

### Integration Tests
```bash
# Start services first
./scripts/start-services.sh

# Run integration tests
./scripts/test-integration.sh
```

### Load Testing
```bash
# Use tools like wrk or hey to test API Gateway
hey -n 1000 -c 10 http://localhost:3000/health
```

## Monitoring and Logging

### Health Checks
Each service provides a `/health` endpoint:
- http://localhost:3001/health (Auth)
- http://localhost:3002/health (Company)
- ... (all other services)

### Logging
Services use structured logging with tracing:
```rust
RUST_LOG="info,accounting=debug" cargo run
```

Logs are written to:
- Console (development)
- Files in `logs/` directory (production)

## Security

### Authentication
- JWT-based authentication with refresh tokens
- Password hashing using bcrypt
- Session management with configurable expiration

### Authorization
- Role-based access control
- Company-level data isolation
- API rate limiting (configurable)

### Data Protection
- Input validation and sanitization
- SQL injection prevention with sqlx
- CORS configuration for web clients

## Performance

### Database Optimization
- Connection pooling with sqlx
- Indexed queries for common operations
- Materialized views for reporting

### Service Communication
- HTTP/REST APIs between services
- Request timeout configuration
- Circuit breaker pattern (recommended)

### Caching
- In-memory caching for frequently accessed data
- Redis integration (optional)

## Troubleshooting

### Common Issues

1. **Service Won't Start**
   ```bash
   # Check if port is already in use
   lsof -i :3001
   
   # Check service logs
   tail -f logs/auth-service.log
   ```

2. **Database Connection Failed**
   ```bash
   # Test database connection
   psql postgresql://user:password@localhost:5432/auth_service
   
   # Check database exists
   ./scripts/setup-databases.sh
   ```

3. **Services Can't Communicate**
   ```bash
   # Check all services are running
   ./scripts/check-services.sh
   
   # Test service connectivity
   curl http://localhost:3001/health
   ```

### Debug Mode
```bash
# Run with debug logging
RUST_LOG="debug" cargo run -p auth-service
```

### Performance Issues
```bash
# Check system resources
htop
iostat -x 1

# Monitor database performance
pg_stat_activity
```

## Contributing

### Code Style
- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Pass `cargo clippy` without warnings
- Write unit tests for new functionality

### Pull Request Process
1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Ensure all checks pass
5. Update documentation
6. Submit pull request

### Indonesian Compliance
When adding new features, ensure:
- Tax calculations follow Indonesian law
- Date/number formatting uses Indonesian standards
- Business validation uses Indonesian rules
- Documentation includes Indonesian context

## Support

### Documentation
- API documentation: `/api/docs` (when running)
- Code documentation: `cargo doc --open`
- Indonesian tax guide: `docs/indonesian-tax.md`

### Community
- GitHub Issues for bugs and feature requests
- Discussions for general questions
- Indonesian business compliance questions welcome

### Commercial Support
For enterprise deployments and Indonesian compliance consulting, contact the maintainers.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Indonesian tax regulations and compliance requirements
- Rust community for excellent ecosystem
- Open source contributors

---

**Note**: This system is designed for Indonesian businesses and includes specific compliance features for Indonesian accounting standards and tax regulations. For use in other countries, modification of tax calculations and validation rules may be required.