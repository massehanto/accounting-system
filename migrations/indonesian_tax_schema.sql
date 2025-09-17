-- indonesian_tax_schema.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

DO $$ BEGIN
    CREATE TYPE tax_type AS ENUM ('PPN', 'PPH21', 'PPH22', 'PPH23', 'PPH25', 'PPH29', 'PBB');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Tax configurations
CREATE TABLE IF NOT EXISTS tax_configurations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL,
    tax_type tax_type NOT NULL,
    tax_rate DECIMAL(5,4) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    effective_date DATE NOT NULL,
    end_date DATE,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Tax transactions
CREATE TABLE IF NOT EXISTS tax_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL,
    tax_type tax_type NOT NULL,
    transaction_date DATE NOT NULL,
    tax_period DATE NOT NULL,
    tax_base_amount DECIMAL(15,2) NOT NULL,
    tax_amount DECIMAL(15,2) NOT NULL,
    tax_invoice_number VARCHAR(100),
    vendor_npwp VARCHAR(20),
    vendor_name VARCHAR(255),
    description TEXT,
    journal_entry_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_tax_configurations_company ON tax_configurations(company_id);
CREATE INDEX IF NOT EXISTS idx_tax_configurations_type ON tax_configurations(company_id, tax_type);
CREATE INDEX IF NOT EXISTS idx_tax_configurations_active ON tax_configurations(company_id, tax_type, is_active);

CREATE INDEX IF NOT EXISTS idx_tax_transactions_company ON tax_transactions(company_id, tax_type);
CREATE INDEX IF NOT EXISTS idx_tax_transactions_date ON tax_transactions(company_id, transaction_date DESC);
CREATE INDEX IF NOT EXISTS idx_tax_transactions_period ON tax_transactions(company_id, tax_period);

-- Triggers
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language plpgsql;

CREATE TRIGGER update_tax_configurations_updated_at BEFORE UPDATE ON tax_configurations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tax_transactions_updated_at BEFORE UPDATE ON tax_transactions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();