// shared/database/src/migrations.rs
use sqlx::PgPool;
use tracing::{info, error};

pub async fn run_migrations(pool: &PgPool, service_name: &str) -> anyhow::Result<()> {
    info!("Running migrations for {}", service_name);
    
    match service_name {
        "auth" => run_auth_migrations(pool).await,
        "company-management" => run_company_migrations(pool).await,
        "chart-of-accounts" => run_chart_migrations(pool).await,
        "general-ledger" => run_ledger_migrations(pool).await,
        "indonesian-tax" => run_tax_migrations(pool).await,
        "accounts-payable" => run_ap_migrations(pool).await,
        "accounts-receivable" => run_ar_migrations(pool).await,
        "inventory-management" => run_inventory_migrations(pool).await,
        _ => {
            error!("Unknown service for migrations: {}", service_name);
            Ok(())
        }
    }
}

// ===== AUTH SERVICE MIGRATIONS =====
async fn run_auth_migrations(pool: &PgPool) -> anyhow::Result<()> {
    info!("Running auth service migrations...");

    // Users table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            full_name VARCHAR(255) NOT NULL,
            company_id UUID NOT NULL,
            is_active BOOLEAN DEFAULT TRUE,
            last_login_at TIMESTAMPTZ,
            failed_login_attempts INTEGER DEFAULT 0,
            locked_until TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Refresh tokens table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            jti VARCHAR(255) PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            expires_at TIMESTAMPTZ NOT NULL,
            is_revoked BOOLEAN DEFAULT FALSE,
            revoked_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // User sessions table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS user_sessions (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            session_token VARCHAR(255) UNIQUE NOT NULL,
            ip_address INET,
            user_agent TEXT,
            expires_at TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_users_company_id ON users(company_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id)")
        .execute(pool).await?;

    info!("Auth migrations completed");
    Ok(())
}

// ===== COMPANY MANAGEMENT MIGRATIONS =====
async fn run_company_migrations(pool: &PgPool) -> anyhow::Result<()> {
    info!("Running company management migrations...");

    // Companies table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS companies (
            id UUID PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            npwp VARCHAR(20) UNIQUE NOT NULL,
            address TEXT NOT NULL,
            phone VARCHAR(50),
            email VARCHAR(255),
            business_type VARCHAR(100),
            registration_number VARCHAR(50),
            tax_registration_number VARCHAR(50),
            website VARCHAR(255),
            logo_url VARCHAR(500),
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Company settings table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS company_settings (
            company_id UUID PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
            fiscal_year_start DATE NOT NULL DEFAULT '2024-01-01',
            default_currency VARCHAR(3) DEFAULT 'IDR',
            timezone VARCHAR(50) DEFAULT 'Asia/Jakarta',
            date_format VARCHAR(20) DEFAULT 'DD/MM/YYYY',
            number_format VARCHAR(20) DEFAULT '1,234.56',
            decimal_places INTEGER DEFAULT 2,
            tax_settings JSONB DEFAULT '{}',
            accounting_method VARCHAR(20) DEFAULT 'ACCRUAL',
            enable_multi_currency BOOLEAN DEFAULT FALSE,
            auto_backup BOOLEAN DEFAULT TRUE,
            backup_frequency VARCHAR(20) DEFAULT 'DAILY',
            notification_settings JSONB DEFAULT '{}',
            report_settings JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Company branches table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS company_branches (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
            branch_code VARCHAR(20) NOT NULL,
            branch_name VARCHAR(255) NOT NULL,
            address TEXT,
            phone VARCHAR(50),
            email VARCHAR(255),
            manager_name VARCHAR(255),
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, branch_code)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_companies_npwp ON companies(npwp)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_company_branches_company_id ON company_branches(company_id)")
        .execute(pool).await?;

    info!("Company management migrations completed");
    Ok(())
}

// ===== CHART OF ACCOUNTS MIGRATIONS =====
async fn run_chart_migrations(pool: &PgPool) -> anyhow::Result<()> {
    info!("Running chart of accounts migrations...");

    // Create enums
    sqlx::query!(
        r#"
        DO $$ BEGIN
            CREATE TYPE account_type AS ENUM ('ASSET', 'LIABILITY', 'EQUITY', 'REVENUE', 'EXPENSE');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        DO $$ BEGIN
            CREATE TYPE account_subtype AS ENUM (
                'CURRENT_ASSET', 'FIXED_ASSET', 'OTHER_ASSET',
                'CURRENT_LIABILITY', 'LONG_TERM_LIABILITY',
                'OWNER_EQUITY', 'RETAINED_EARNINGS',
                'OPERATING_REVENUE', 'NON_OPERATING_REVENUE',
                'COST_OF_GOODS_SOLD', 'OPERATING_EXPENSE', 'NON_OPERATING_EXPENSE'
            );
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#
    )
    .execute(pool)
    .await?;

    // Accounts table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            account_code VARCHAR(10) NOT NULL,
            account_name VARCHAR(255) NOT NULL,
            account_type account_type NOT NULL,
            account_subtype account_subtype,
            parent_account_id UUID,
            normal_balance VARCHAR(10) DEFAULT 'DEBIT',
            description TEXT,
            is_system BOOLEAN DEFAULT FALSE,
            is_active BOOLEAN DEFAULT TRUE,
            is_reconcilable BOOLEAN DEFAULT FALSE,
            bank_account_number VARCHAR(50),
            bank_name VARCHAR(255),
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, account_code),
            FOREIGN KEY (parent_account_id) REFERENCES accounts(id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Account templates table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS account_templates (
            id UUID PRIMARY KEY,
            template_name VARCHAR(100) NOT NULL,
            description TEXT,
            country_code VARCHAR(3) DEFAULT 'IDN',
            industry VARCHAR(100),
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Account template items table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS account_template_items (
            id UUID PRIMARY KEY,
            template_id UUID NOT NULL REFERENCES account_templates(id) ON DELETE CASCADE,
            account_code VARCHAR(10) NOT NULL,
            account_name VARCHAR(255) NOT NULL,
            account_type account_type NOT NULL,
            account_subtype account_subtype,
            parent_code VARCHAR(10),
            normal_balance VARCHAR(10) DEFAULT 'DEBIT',
            sort_order INTEGER DEFAULT 0
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_accounts_company_id ON accounts(company_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(account_type)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_accounts_parent ON accounts(parent_account_id)")
        .execute(pool).await?;

    info!("Chart of accounts migrations completed");
    Ok(())
}

// ===== GENERAL LEDGER MIGRATIONS =====
async fn run_ledger_migrations(pool: &PgPool) -> anyhow::Result<()> {
    info!("Running general ledger migrations...");

    // Create enums
    sqlx::query!(
        r#"
        DO $$ BEGIN
            CREATE TYPE journal_entry_status AS ENUM ('DRAFT', 'PENDING_APPROVAL', 'APPROVED', 'POSTED', 'CANCELLED');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#
    )
    .execute(pool)
    .await?;

    // Journal entries table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS journal_entries (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            entry_number VARCHAR(50) UNIQUE NOT NULL,
            entry_date DATE NOT NULL,
            description TEXT,
            reference VARCHAR(100),
            total_debit DECIMAL(15,2) NOT NULL,
            total_credit DECIMAL(15,2) NOT NULL,
            status journal_entry_status DEFAULT 'DRAFT',
            is_posted BOOLEAN DEFAULT FALSE,
            is_recurring BOOLEAN DEFAULT FALSE,
            recurring_template_id UUID,
            source_document_type VARCHAR(50),
            source_document_id UUID,
            created_by UUID NOT NULL,
            approved_by UUID,
            posted_by UUID,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            approved_at TIMESTAMPTZ,
            posted_at TIMESTAMPTZ,
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Journal entry lines table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS journal_entry_lines (
            id UUID PRIMARY KEY,
            journal_entry_id UUID NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
            account_id UUID NOT NULL,
            description TEXT,
            debit_amount DECIMAL(15,2) DEFAULT 0,
            credit_amount DECIMAL(15,2) DEFAULT 0,
            line_number INTEGER NOT NULL,
            department VARCHAR(100),
            project_code VARCHAR(50),
            cost_center VARCHAR(50),
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Audit logs table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id UUID PRIMARY KEY,
            table_name VARCHAR(50) NOT NULL,
            record_id UUID NOT NULL,
            action VARCHAR(50) NOT NULL,
            old_values JSONB,
            new_values JSONB,
            user_id UUID NOT NULL,
            ip_address INET,
            user_agent TEXT,
            timestamp TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Recurring journal templates table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS recurring_journal_templates (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            template_name VARCHAR(255) NOT NULL,
            description TEXT,
            frequency VARCHAR(20) NOT NULL, -- MONTHLY, QUARTERLY, YEARLY
            next_run_date DATE NOT NULL,
            is_active BOOLEAN DEFAULT TRUE,
            created_by UUID NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Account balances materialized view/table for performance
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS account_balances (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            account_id UUID NOT NULL,
            balance_date DATE NOT NULL,
            beginning_balance DECIMAL(15,2) DEFAULT 0,
            debit_amount DECIMAL(15,2) DEFAULT 0,
            credit_amount DECIMAL(15,2) DEFAULT 0,
            ending_balance DECIMAL(15,2) DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, account_id, balance_date)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create function for generating entry numbers
    sqlx::query!(
        r#"
        CREATE OR REPLACE FUNCTION generate_entry_number(company_uuid UUID, entry_date DATE)
        RETURNS VARCHAR(50) AS $$
        DECLARE
            year_part VARCHAR(4);
            month_part VARCHAR(2);
            sequence_num INTEGER;
            entry_number VARCHAR(50);
        BEGIN
            year_part := EXTRACT(YEAR FROM entry_date)::VARCHAR;
            month_part := LPAD(EXTRACT(MONTH FROM entry_date)::VARCHAR, 2, '0');
            
            SELECT COALESCE(MAX(CAST(SUBSTRING(entry_number FROM 'JE-\d{4}\d{2}-(\d+)') AS INTEGER)), 0) + 1
            INTO sequence_num
            FROM journal_entries 
            WHERE company_id = company_uuid 
            AND EXTRACT(YEAR FROM entry_date) = EXTRACT(YEAR FROM entry_date)
            AND EXTRACT(MONTH FROM entry_date) = EXTRACT(MONTH FROM entry_date);
            
            entry_number := 'JE-' || year_part || month_part || '-' || LPAD(sequence_num::VARCHAR, 6, '0');
            
            RETURN entry_number;
        END;
        $$ LANGUAGE plpgsql;
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_journal_entries_company_id ON journal_entries(company_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_journal_entries_date ON journal_entries(entry_date)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_journal_entries_status ON journal_entries(status)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_journal_entry_lines_entry_id ON journal_entry_lines(journal_entry_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_journal_entry_lines_account_id ON journal_entry_lines(account_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_audit_logs_table_record ON audit_logs(table_name, record_id)")
        .execute(pool).await?;

    info!("General ledger migrations completed");
    Ok(())
}

// ===== INDONESIAN TAX MIGRATIONS =====
async fn run_tax_migrations(pool: &PgPool) -> anyhow::Result<()> {
    info!("Running Indonesian tax migrations...");

    // Create tax type enum
    sqlx::query!(
        r#"
        DO $$ BEGIN
            CREATE TYPE tax_type AS ENUM ('PPN', 'PPH21', 'PPH22', 'PPH23', 'PPH25', 'PPH29', 'PBB');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#
    )
    .execute(pool)
    .await?;

    // Tax configurations table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS tax_configurations (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            tax_type tax_type NOT NULL,
            tax_rate DECIMAL(5,2) NOT NULL,
            is_active BOOLEAN DEFAULT TRUE,
            effective_date DATE NOT NULL,
            end_date DATE,
            description TEXT,
            calculation_method VARCHAR(50) DEFAULT 'PERCENTAGE',
            minimum_amount DECIMAL(15,2) DEFAULT 0,
            maximum_amount DECIMAL(15,2),
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Tax transactions table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS tax_transactions (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            tax_type tax_type NOT NULL,
            transaction_date DATE NOT NULL,
            tax_period DATE NOT NULL,
            tax_base_amount DECIMAL(15,2) NOT NULL,
            tax_amount DECIMAL(15,2) NOT NULL,
            tax_invoice_number VARCHAR(50),
            vendor_npwp VARCHAR(20),
            vendor_name VARCHAR(255),
            customer_npwp VARCHAR(20),
            customer_name VARCHAR(255),
            description TEXT,
            journal_entry_id UUID,
            source_document_type VARCHAR(50),
            source_document_id UUID,
            is_reversed BOOLEAN DEFAULT FALSE,
            reversal_reason TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // E-Faktur data table (for Indonesian VAT)
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS efaktur_data (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            tax_transaction_id UUID NOT NULL REFERENCES tax_transactions(id) ON DELETE CASCADE,
            faktur_number VARCHAR(50) NOT NULL,
            faktur_date DATE NOT NULL,
            vendor_npwp VARCHAR(20) NOT NULL,
            vendor_name VARCHAR(255) NOT NULL,
            dpp_amount DECIMAL(15,2) NOT NULL,
            ppn_amount DECIMAL(15,2) NOT NULL,
            ppnbm_amount DECIMAL(15,2) DEFAULT 0,
            referensi VARCHAR(100),
            status VARCHAR(20) DEFAULT 'DRAFT',
            uploaded_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, faktur_number)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Tax periods table (for reporting)
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS tax_periods (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            tax_type tax_type NOT NULL,
            period_year INTEGER NOT NULL,
            period_month INTEGER,
            period_quarter INTEGER,
            status VARCHAR(20) DEFAULT 'OPEN',
            closed_at TIMESTAMPTZ,
            closed_by UUID,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, tax_type, period_year, period_month),
            UNIQUE(company_id, tax_type, period_year, period_quarter)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_tax_configurations_company_type ON tax_configurations(company_id, tax_type)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_tax_transactions_company_id ON tax_transactions(company_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_tax_transactions_period ON tax_transactions(tax_period)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_tax_transactions_type ON tax_transactions(tax_type)")
        .execute(pool).await?;

    info!("Indonesian tax migrations completed");
    Ok(())
}

// ===== ACCOUNTS PAYABLE MIGRATIONS =====
async fn run_ap_migrations(pool: &PgPool) -> anyhow::Result<()> {
    info!("Running accounts payable migrations...");

    // Create invoice status enum
    sqlx::query!(
        r#"
        DO $$ BEGIN
            CREATE TYPE invoice_status AS ENUM ('DRAFT', 'PENDING', 'APPROVED', 'PAID', 'CANCELLED');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#
    )
    .execute(pool)
    .await?;

    // Vendors table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS vendors (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            vendor_code VARCHAR(20) NOT NULL,
            vendor_name VARCHAR(255) NOT NULL,
            npwp VARCHAR(20),
            address TEXT,
            phone VARCHAR(50),
            email VARCHAR(255),
            contact_person VARCHAR(255),
            payment_terms INTEGER DEFAULT 30,
            bank_name VARCHAR(255),
            bank_account_number VARCHAR(50),
            bank_account_name VARCHAR(255),
            currency VARCHAR(3) DEFAULT 'IDR',
            is_active BOOLEAN DEFAULT TRUE,
            notes TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, vendor_code)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Vendor invoices table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS vendor_invoices (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            vendor_id UUID NOT NULL REFERENCES vendors(id),
            invoice_number VARCHAR(50) NOT NULL,
            invoice_date DATE NOT NULL,
            due_date DATE NOT NULL,
            subtotal DECIMAL(15,2) NOT NULL,
            tax_amount DECIMAL(15,2) DEFAULT 0,
            discount_amount DECIMAL(15,2) DEFAULT 0,
            total_amount DECIMAL(15,2) NOT NULL,
            paid_amount DECIMAL(15,2) DEFAULT 0,
            status invoice_status DEFAULT 'DRAFT',
            description TEXT,
            purchase_order_number VARCHAR(50),
            received_date DATE,
            journal_entry_id UUID,
            currency VARCHAR(3) DEFAULT 'IDR',
            exchange_rate DECIMAL(10,4) DEFAULT 1.0000,
            created_by UUID NOT NULL,
            approved_by UUID,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, vendor_id, invoice_number)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Vendor payments table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS vendor_payments (
            id UUID PRIMARY KEY,
            invoice_id UUID NOT NULL REFERENCES vendor_invoices(id),
            company_id UUID NOT NULL,
            payment_number VARCHAR(50) NOT NULL,
            payment_amount DECIMAL(15,2) NOT NULL,
            payment_date DATE NOT NULL,
            payment_method VARCHAR(50) NOT NULL,
            bank_account_id UUID,
            bank_account_number VARCHAR(50),
            check_number VARCHAR(50),
            payment_reference VARCHAR(255),
            notes TEXT,
            is_reversed BOOLEAN DEFAULT FALSE,
            reversal_reason TEXT,
            reversed_by UUID,
            reversed_at TIMESTAMPTZ,
            journal_entry_id UUID,
            created_by UUID NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Vendor invoice line items table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS vendor_invoice_lines (
            id UUID PRIMARY KEY,
            invoice_id UUID NOT NULL REFERENCES vendor_invoices(id) ON DELETE CASCADE,
            line_number INTEGER NOT NULL,
            description TEXT NOT NULL,
            quantity DECIMAL(15,4) DEFAULT 1,
            unit_price DECIMAL(15,2) NOT NULL,
            line_amount DECIMAL(15,2) NOT NULL,
            account_id UUID,
            department VARCHAR(100),
            project_code VARCHAR(50),
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Purchase orders table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS purchase_orders (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            vendor_id UUID NOT NULL REFERENCES vendors(id),
            po_number VARCHAR(50) NOT NULL,
            po_date DATE NOT NULL,
            expected_delivery_date DATE,
            subtotal DECIMAL(15,2) NOT NULL,
            tax_amount DECIMAL(15,2) DEFAULT 0,
            total_amount DECIMAL(15,2) NOT NULL,
            status VARCHAR(20) DEFAULT 'PENDING',
            notes TEXT,
            created_by UUID NOT NULL,
            approved_by UUID,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, po_number)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_vendors_company_id ON vendors(company_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_vendors_npwp ON vendors(npwp)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_vendor_invoices_company_id ON vendor_invoices(company_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_vendor_invoices_vendor_id ON vendor_invoices(vendor_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_vendor_invoices_status ON vendor_invoices(status)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_vendor_invoices_due_date ON vendor_invoices(due_date)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_vendor_payments_invoice_id ON vendor_payments(invoice_id)")
        .execute(pool).await?;

    info!("Accounts payable migrations completed");
    Ok(())
}

// ===== ACCOUNTS RECEIVABLE MIGRATIONS =====
async fn run_ar_migrations(pool: &PgPool) -> anyhow::Result<()> {
    info!("Running accounts receivable migrations...");

    // Customers table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS customers (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            customer_code VARCHAR(20) NOT NULL,
            customer_name VARCHAR(255) NOT NULL,
            npwp VARCHAR(20),
            address TEXT,
            phone VARCHAR(50),
            email VARCHAR(255),
            contact_person VARCHAR(255),
            credit_limit DECIMAL(15,2) DEFAULT 0,
            payment_terms INTEGER DEFAULT 30,
            bank_name VARCHAR(255),
            bank_account_number VARCHAR(50),
            bank_account_name VARCHAR(255),
            currency VARCHAR(3) DEFAULT 'IDR',
            price_level VARCHAR(50) DEFAULT 'STANDARD',
            sales_rep VARCHAR(255),
            territory VARCHAR(100),
            industry VARCHAR(100),
            is_active BOOLEAN DEFAULT TRUE,
            notes TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, customer_code)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Customer invoices table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS customer_invoices (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            customer_id UUID NOT NULL REFERENCES customers(id),
            invoice_number VARCHAR(50) NOT NULL,
            invoice_date DATE NOT NULL,
            due_date DATE NOT NULL,
            subtotal DECIMAL(15,2) NOT NULL,
            tax_amount DECIMAL(15,2) DEFAULT 0,
            discount_amount DECIMAL(15,2) DEFAULT 0,
            shipping_amount DECIMAL(15,2) DEFAULT 0,
            total_amount DECIMAL(15,2) NOT NULL,
            paid_amount DECIMAL(15,2) DEFAULT 0,
            status invoice_status DEFAULT 'DRAFT',
            description TEXT,
            sales_order_number VARCHAR(50),
            delivery_date DATE,
            journal_entry_id UUID,
            currency VARCHAR(3) DEFAULT 'IDR',
            exchange_rate DECIMAL(10,4) DEFAULT 1.0000,
            created_by UUID NOT NULL,
            approved_by UUID,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, customer_id, invoice_number)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Customer payments table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS customer_payments (
            id UUID PRIMARY KEY,
            invoice_id UUID NOT NULL REFERENCES customer_invoices(id),
            company_id UUID NOT NULL,
            payment_number VARCHAR(50) NOT NULL,
            payment_amount DECIMAL(15,2) NOT NULL,
            payment_date DATE NOT NULL,
            payment_method VARCHAR(50) NOT NULL,
            bank_account_id UUID,
            bank_account_number VARCHAR(50),
            check_number VARCHAR(50),
            payment_reference VARCHAR(255),
            notes TEXT,
            is_reversed BOOLEAN DEFAULT FALSE,
            reversal_reason TEXT,
            reversed_by UUID,
            reversed_at TIMESTAMPTZ,
            journal_entry_id UUID,
            created_by UUID NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Customer invoice line items table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS customer_invoice_lines (
            id UUID PRIMARY KEY,
            invoice_id UUID NOT NULL REFERENCES customer_invoices(id) ON DELETE CASCADE,
            line_number INTEGER NOT NULL,
            description TEXT NOT NULL,
            quantity DECIMAL(15,4) DEFAULT 1,
            unit_price DECIMAL(15,2) NOT NULL,
            line_amount DECIMAL(15,2) NOT NULL,
            product_id UUID,
            account_id UUID,
            department VARCHAR(100),
            project_code VARCHAR(50),
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Sales orders table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS sales_orders (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            customer_id UUID NOT NULL REFERENCES customers(id),
            so_number VARCHAR(50) NOT NULL,
            so_date DATE NOT NULL,
            expected_delivery_date DATE,
            subtotal DECIMAL(15,2) NOT NULL,
            tax_amount DECIMAL(15,2) DEFAULT 0,
            total_amount DECIMAL(15,2) NOT NULL,
            status VARCHAR(20) DEFAULT 'PENDING',
            notes TEXT,
            created_by UUID NOT NULL,
            approved_by UUID,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, so_number)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Customer credit applications table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS customer_credit_applications (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            customer_id UUID NOT NULL REFERENCES customers(id),
            requested_limit DECIMAL(15,2) NOT NULL,
            approved_limit DECIMAL(15,2),
            application_date DATE NOT NULL,
            status VARCHAR(20) DEFAULT 'PENDING',
            reviewed_by UUID,
            reviewed_at TIMESTAMPTZ,
            notes TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_customers_company_id ON customers(company_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_customers_npwp ON customers(npwp)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_customer_invoices_company_id ON customer_invoices(company_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_customer_invoices_customer_id ON customer_invoices(customer_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_customer_invoices_status ON customer_invoices(status)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_customer_invoices_due_date ON customer_invoices(due_date)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_customer_payments_invoice_id ON customer_payments(invoice_id)")
        .execute(pool).await?;

    info!("Accounts receivable migrations completed");
    Ok(())
}

// ===== INVENTORY MANAGEMENT MIGRATIONS =====
async fn run_inventory_migrations(pool: &PgPool) -> anyhow::Result<()> {
    info!("Running inventory management migrations...");

    // Create item type enum
    sqlx::query!(
        r#"
        DO $$ BEGIN
            CREATE TYPE item_type AS ENUM ('RAW', 'FINISHED', 'SERVICE');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#
    )
    .execute(pool)
    .await?;

    // Item categories table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS item_categories (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            category_code VARCHAR(20) NOT NULL,
            category_name VARCHAR(255) NOT NULL,
            parent_category_id UUID,
            description TEXT,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, category_code),
            FOREIGN KEY (parent_category_id) REFERENCES item_categories(id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Inventory items table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS inventory_items (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            item_code VARCHAR(50) NOT NULL,
            item_name VARCHAR(255) NOT NULL,
            description TEXT,
            item_type item_type NOT NULL,
            category_id UUID REFERENCES item_categories(id),
            unit_of_measure VARCHAR(10) NOT NULL,
            unit_cost DECIMAL(15,2) NOT NULL,
            selling_price DECIMAL(15,2) NOT NULL,
            quantity_on_hand DECIMAL(15,4) DEFAULT 0,
            quantity_committed DECIMAL(15,4) DEFAULT 0,
            quantity_available DECIMAL(15,4) DEFAULT 0,
            reorder_level DECIMAL(15,4) DEFAULT 0,
            maximum_level DECIMAL(15,4),
            lead_time_days INTEGER DEFAULT 0,
            supplier_item_code VARCHAR(50),
            barcode VARCHAR(100),
            location VARCHAR(100),
            bin_location VARCHAR(50),
            weight DECIMAL(10,3),
            dimensions VARCHAR(100),
            is_serialized BOOLEAN DEFAULT FALSE,
            is_lot_tracked BOOLEAN DEFAULT FALSE,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, item_code)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Inventory transactions table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS inventory_transactions (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            item_id UUID NOT NULL REFERENCES inventory_items(id),
            transaction_type VARCHAR(20) NOT NULL CHECK (transaction_type IN ('IN', 'OUT', 'ADJUSTMENT', 'TRANSFER')),
            transaction_date DATE NOT NULL,
            quantity DECIMAL(15,4) NOT NULL,
            unit_cost DECIMAL(15,2) NOT NULL,
            total_cost DECIMAL(15,2) NOT NULL,
            reference VARCHAR(255),
            source_document_type VARCHAR(50),
            source_document_id UUID,
            location_from VARCHAR(100),
            location_to VARCHAR(100),
            lot_number VARCHAR(50),
            serial_number VARCHAR(100),
            journal_entry_id UUID,
            created_by UUID NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Inventory adjustments table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS inventory_adjustments (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            adjustment_number VARCHAR(50) NOT NULL,
            adjustment_date DATE NOT NULL,
            reason VARCHAR(255) NOT NULL,
            status VARCHAR(20) DEFAULT 'DRAFT',
            total_value DECIMAL(15,2) DEFAULT 0,
            notes TEXT,
            created_by UUID NOT NULL,
            approved_by UUID,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, adjustment_number)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Inventory adjustment lines table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS inventory_adjustment_lines (
            id UUID PRIMARY KEY,
            adjustment_id UUID NOT NULL REFERENCES inventory_adjustments(id) ON DELETE CASCADE,
            item_id UUID NOT NULL REFERENCES inventory_items(id),
            counted_quantity DECIMAL(15,4) NOT NULL,
            system_quantity DECIMAL(15,4) NOT NULL,
            adjustment_quantity DECIMAL(15,4) NOT NULL,
            unit_cost DECIMAL(15,2) NOT NULL,
            adjustment_value DECIMAL(15,2) NOT NULL,
            notes TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Inventory locations table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS inventory_locations (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            location_code VARCHAR(20) NOT NULL,
            location_name VARCHAR(255) NOT NULL,
            location_type VARCHAR(50) DEFAULT 'WAREHOUSE',
            address TEXT,
            is_default BOOLEAN DEFAULT FALSE,
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, location_code)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Inventory valuation table (for different costing methods)
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS inventory_valuations (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            item_id UUID NOT NULL REFERENCES inventory_items(id),
            valuation_date DATE NOT NULL,
            costing_method VARCHAR(20) NOT NULL, -- FIFO, LIFO, AVERAGE, STANDARD
            unit_cost DECIMAL(15,2) NOT NULL,
            quantity DECIMAL(15,4) NOT NULL,
            total_value DECIMAL(15,2) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, item_id, valuation_date, costing_method)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Bill of materials table
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS bill_of_materials (
            id UUID PRIMARY KEY,
            company_id UUID NOT NULL,
            finished_item_id UUID NOT NULL REFERENCES inventory_items(id),
            component_item_id UUID NOT NULL REFERENCES inventory_items(id),
            quantity_per_unit DECIMAL(15,4) NOT NULL,
            unit_cost DECIMAL(15,2),
            is_active BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(company_id, finished_item_id, component_item_id)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_inventory_items_company_id ON inventory_items(company_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_inventory_items_category ON inventory_items(category_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_inventory_items_type ON inventory_items(item_type)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_inventory_transactions_item_id ON inventory_transactions(item_id)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_inventory_transactions_date ON inventory_transactions(transaction_date)")
        .execute(pool).await?;
    sqlx::query!("CREATE INDEX IF NOT EXISTS idx_inventory_transactions_type ON inventory_transactions(transaction_type)")
        .execute(pool).await?;

    info!("Inventory management migrations completed");
    Ok(())
}

// ===== HELPER FUNCTIONS =====

/// Function to create default chart of accounts for Indonesian companies
pub async fn create_default_indonesian_chart_of_accounts(
    pool: &PgPool,
    company_id: uuid::Uuid,
) -> anyhow::Result<()> {
    info!("Creating default Indonesian chart of accounts for company {}", company_id);

    let default_accounts = vec![
        // ASSETS
        ("1100", "Kas", "ASSET", "CURRENT_ASSET", "DEBIT"),
        ("1110", "Bank", "ASSET", "CURRENT_ASSET", "DEBIT"),
        ("1200", "Piutang Dagang", "ASSET", "CURRENT_ASSET", "DEBIT"),
        ("1210", "Piutang Lain-lain", "ASSET", "CURRENT_ASSET", "DEBIT"),
        ("1300", "Persediaan", "ASSET", "CURRENT_ASSET", "DEBIT"),
        ("1400", "Biaya Dibayar Dimuka", "ASSET", "CURRENT_ASSET", "DEBIT"),
        ("1500", "Tanah", "ASSET", "FIXED_ASSET", "DEBIT"),
        ("1510", "Bangunan", "ASSET", "FIXED_ASSET", "DEBIT"),
        ("1520", "Mesin dan Peralatan", "ASSET", "FIXED_ASSET", "DEBIT"),
        ("1530", "Kendaraan", "ASSET", "FIXED_ASSET", "DEBIT"),
        ("1540", "Peralatan Kantor", "ASSET", "FIXED_ASSET", "DEBIT"),
        ("1590", "Akumulasi Penyusutan", "ASSET", "FIXED_ASSET", "CREDIT"),
        
        // LIABILITIES
        ("2100", "Hutang Dagang", "LIABILITY", "CURRENT_LIABILITY", "CREDIT"),
        ("2110", "Hutang PPN", "LIABILITY", "CURRENT_LIABILITY", "CREDIT"),
        ("2120", "Hutang PPh 21", "LIABILITY", "CURRENT_LIABILITY", "CREDIT"),
        ("2121", "Hutang PPh 22", "LIABILITY", "CURRENT_LIABILITY", "CREDIT"),
        ("2122", "Hutang PPh 23", "LIABILITY", "CURRENT_LIABILITY", "CREDIT"),
        ("2130", "Hutang Gaji", "LIABILITY", "CURRENT_LIABILITY", "CREDIT"),
        ("2140", "Hutang Lain-lain", "LIABILITY", "CURRENT_LIABILITY", "CREDIT"),
        ("2200", "Hutang Bank Jangka Panjang", "LIABILITY", "LONG_TERM_LIABILITY", "CREDIT"),
        
        // EQUITY
        ("3100", "Modal Disetor", "EQUITY", "OWNER_EQUITY", "CREDIT"),
        ("3200", "Laba Ditahan", "EQUITY", "RETAINED_EARNINGS", "CREDIT"),
        ("3300", "Laba Tahun Berjalan", "EQUITY", "RETAINED_EARNINGS", "CREDIT"),
        
        // REVENUE
        ("4100", "Pendapatan Penjualan", "REVENUE", "OPERATING_REVENUE", "CREDIT"),
        ("4200", "Pendapatan Lain-lain", "REVENUE", "NON_OPERATING_REVENUE", "CREDIT"),
        
        // EXPENSES
        ("5100", "Harga Pokok Penjualan", "EXPENSE", "COST_OF_GOODS_SOLD", "DEBIT"),
        ("6100", "Beban Gaji dan Upah", "EXPENSE", "OPERATING_EXPENSE", "DEBIT"),
        ("6110", "Beban Sewa", "EXPENSE", "OPERATING_EXPENSE", "DEBIT"),
        ("6120", "Beban Listrik", "EXPENSE", "OPERATING_EXPENSE", "DEBIT"),
        ("6130", "Beban Telepon", "EXPENSE", "OPERATING_EXPENSE", "DEBIT"),
        ("6140", "Beban Pemasaran", "EXPENSE", "OPERATING_EXPENSE", "DEBIT"),
        ("6150", "Beban Administrasi", "EXPENSE", "OPERATING_EXPENSE", "DEBIT"),
        ("6160", "Beban Penyusutan", "EXPENSE", "OPERATING_EXPENSE", "DEBIT"),
        ("6170", "Beban Bunga", "EXPENSE", "NON_OPERATING_EXPENSE", "DEBIT"),
        ("6180", "Beban Lain-lain", "EXPENSE", "NON_OPERATING_EXPENSE", "DEBIT"),
    ];

    for (code, name, acc_type, subtype, normal_balance) in default_accounts {
        sqlx::query!(
            r#"
            INSERT INTO accounts (id, company_id, account_code, account_name, account_type, account_subtype, normal_balance, is_system, is_active)
            VALUES ($1, $2, $3, $4, $5::account_type, $6::account_subtype, $7, true, true)
            ON CONFLICT (company_id, account_code) DO NOTHING
            "#,
            uuid::Uuid::new_v4(),
            company_id,
            code,
            name,
            acc_type,
            subtype,
            normal_balance
        )
        .execute(pool)
        .await?;
    }

    info!("Default Indonesian chart of accounts created successfully");
    Ok(())
}

/// Function to create default tax configurations for Indonesian companies
pub async fn create_default_indonesian_tax_config(
    pool: &PgPool,
    company_id: uuid::Uuid,
) -> anyhow::Result<()> {
    info!("Creating default Indonesian tax configurations for company {}", company_id);

    let default_taxes = vec![
        ("PPN", 11.0, "Pajak Pertambahan Nilai"),
        ("PPH21", 5.0, "PPh 21 - Pajak Penghasilan Pasal 21"),
        ("PPH22", 1.5, "PPh 22 - Pajak Penghasilan Pasal 22"),
        ("PPH23", 2.0, "PPh 23 - Pajak Penghasilan Pasal 23"),
        ("PPH25", 1.0, "PPh 25 - Angsuran Pajak Penghasilan"),
        ("PBB", 0.5, "Pajak Bumi dan Bangunan"),
    ];

    for (tax_type, rate, description) in default_taxes {
        sqlx::query!(
            r#"
            INSERT INTO tax_configurations (id, company_id, tax_type, tax_rate, effective_date, description, is_active)
            VALUES ($1, $2, $3::tax_type, $4, CURRENT_DATE, $5, true)
            ON CONFLICT DO NOTHING
            "#,
            uuid::Uuid::new_v4(),
            company_id,
            tax_type,
            rate,
            description
        )
        .execute(pool)
        .await?;
    }

    info!("Default Indonesian tax configurations created successfully");
    Ok(())
}

/// Function to run all migrations for a new company setup
pub async fn setup_new_company_data(
    pool: &PgPool,
    company_id: uuid::Uuid,
) -> anyhow::Result<()> {
    info!("Setting up new company data for company {}", company_id);
    
    // Create default chart of accounts
    create_default_indonesian_chart_of_accounts(pool, company_id).await?;
    
    // Create default tax configurations
    create_default_indonesian_tax_config(pool, company_id).await?;
    
    // Create default inventory location
    sqlx::query!(
        r#"
        INSERT INTO inventory_locations (id, company_id, location_code, location_name, location_type, is_default, is_active)
        VALUES ($1, $2, 'MAIN', 'Gudang Utama', 'WAREHOUSE', true, true)
        ON CONFLICT (company_id, location_code) DO NOTHING
        "#,
        uuid::Uuid::new_v4(),
        company_id,
        
    )
    .execute(pool)
    .await?;
    
    info!("New company data setup completed for company {}", company_id);
    Ok(())
}