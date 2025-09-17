-- chart_of_accounts_schema.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create ENUM types
DO $$ BEGIN
    CREATE TYPE account_type AS ENUM ('ASSET', 'LIABILITY', 'EQUITY', 'REVENUE', 'EXPENSE');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

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

-- Chart of Accounts
CREATE TABLE IF NOT EXISTS accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL,
    account_code VARCHAR(20) NOT NULL,
    account_name VARCHAR(255) NOT NULL,
    account_type account_type NOT NULL,
    account_subtype account_subtype,
    parent_account_id UUID REFERENCES accounts(id),
    normal_balance VARCHAR(10) NOT NULL CHECK (normal_balance IN ('DEBIT', 'CREDIT')),
    is_system BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(company_id, account_code)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_accounts_company_code ON accounts(company_id, account_code);
CREATE INDEX IF NOT EXISTS idx_accounts_company_type ON accounts(company_id, account_type);
CREATE INDEX IF NOT EXISTS idx_accounts_parent ON accounts(parent_account_id);
CREATE INDEX IF NOT EXISTS idx_accounts_active ON accounts(company_id, is_active);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language plpgsql;

CREATE TRIGGER update_accounts_updated_at BEFORE UPDATE ON accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to set normal balance based on account type
CREATE OR REPLACE FUNCTION set_normal_balance()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.normal_balance IS NULL THEN
        CASE NEW.account_type
            WHEN 'ASSET' THEN NEW.normal_balance := 'DEBIT';
            WHEN 'EXPENSE' THEN NEW.normal_balance := 'DEBIT';
            WHEN 'LIABILITY' THEN NEW.normal_balance := 'CREDIT';
            WHEN 'EQUITY' THEN NEW.normal_balance := 'CREDIT';
            WHEN 'REVENUE' THEN NEW.normal_balance := 'CREDIT';
            ELSE NEW.normal_balance := 'DEBIT';
        END CASE;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_account_normal_balance
    BEFORE INSERT OR UPDATE ON accounts
    FOR EACH ROW
    EXECUTE FUNCTION set_normal_balance();