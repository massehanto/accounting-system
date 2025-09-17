-- general_ledger_schema.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Journal entry status enum
DO $$ BEGIN
    CREATE TYPE journal_entry_status AS ENUM ('DRAFT', 'PENDING_APPROVAL', 'APPROVED', 'POSTED', 'CANCELLED');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create sequence for journal entry numbering
CREATE SEQUENCE IF NOT EXISTS journal_entry_sequence 
    START WITH 1 INCREMENT BY 1 MINVALUE 1 CACHE 1;

-- Journal entries
CREATE TABLE IF NOT EXISTS journal_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_id UUID NOT NULL,
    entry_number VARCHAR(50) NOT NULL,
    entry_date DATE NOT NULL,
    description TEXT,
    reference VARCHAR(100),
    total_debit DECIMAL(15,2) NOT NULL DEFAULT 0,
    total_credit DECIMAL(15,2) NOT NULL DEFAULT 0,
    status journal_entry_status DEFAULT 'DRAFT',
    is_posted BOOLEAN DEFAULT false,
    created_by UUID NOT NULL,
    approved_by UUID,
    posted_by UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    approved_at TIMESTAMP WITH TIME ZONE,
    posted_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(company_id, entry_number),
    CONSTRAINT balanced_entry CHECK (total_debit = total_credit),
    CONSTRAINT status_posted_sync CHECK (
        (status = 'POSTED' AND is_posted = true) OR 
        (status != 'POSTED' AND is_posted = false)
    )
);

-- Journal entry lines
CREATE TABLE IF NOT EXISTS journal_entry_lines (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    journal_entry_id UUID NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    account_id UUID NOT NULL,
    description TEXT,
    debit_amount DECIMAL(15,2) DEFAULT 0 CHECK (debit_amount >= 0),
    credit_amount DECIMAL(15,2) DEFAULT 0 CHECK (credit_amount >= 0),
    line_number INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT debit_or_credit CHECK (
        (debit_amount > 0 AND credit_amount = 0) OR 
        (credit_amount > 0 AND debit_amount = 0)
    )
);

-- Audit logs
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    table_name VARCHAR(255) NOT NULL,
    record_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    user_id UUID NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_journal_entries_company_date ON journal_entries(company_id, entry_date DESC);
CREATE INDEX IF NOT EXISTS idx_journal_entries_status ON journal_entries(company_id, status);
CREATE INDEX IF NOT EXISTS idx_journal_entries_posted ON journal_entries(company_id, is_posted);
CREATE INDEX IF NOT EXISTS idx_journal_entries_number ON journal_entries(company_id, entry_number);
CREATE INDEX IF NOT EXISTS idx_journal_entries_created_by ON journal_entries(created_by);

CREATE INDEX IF NOT EXISTS idx_journal_entry_lines_entry ON journal_entry_lines(journal_entry_id);
CREATE INDEX IF NOT EXISTS idx_journal_entry_lines_account ON journal_entry_lines(account_id);

CREATE INDEX IF NOT EXISTS idx_audit_logs_record ON audit_logs(table_name, record_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);

-- Functions
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language plpgsql;

CREATE TRIGGER update_journal_entries_updated_at BEFORE UPDATE ON journal_entries
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to generate sequential journal entry numbers
CREATE OR REPLACE FUNCTION generate_entry_number(p_company_id UUID, p_entry_date DATE)
RETURNS TEXT AS $$
DECLARE
    year_part INTEGER;
    next_number INTEGER;
    entry_number TEXT;
BEGIN
    year_part := EXTRACT(YEAR FROM p_entry_date);
    
    SELECT COALESCE(MAX(
        CASE 
            WHEN entry_number ~ ('^JE-' || year_part || '-\d+$') THEN 
                CAST(SUBSTRING(entry_number FROM ('^JE-' || year_part || '-(\d+)$')) AS INTEGER)
            ELSE 0
        END
    ), 0) + 1
    INTO next_number
    FROM journal_entries 
    WHERE company_id = p_company_id 
    AND EXTRACT(YEAR FROM entry_date) = year_part;
    
    entry_number := 'JE-' || year_part || '-' || LPAD(next_number::TEXT, 6, '0');
    
    RETURN entry_number;
END;
$$ LANGUAGE plpgsql;

-- Status validation function
CREATE OR REPLACE FUNCTION validate_status_transition()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        RETURN NEW;
    END IF;
    
    IF OLD.is_posted = true AND NEW.is_posted = true AND OLD.status != NEW.status THEN
        RAISE EXCEPTION 'Cannot change status of posted journal entry';
    END IF;
    
    IF OLD.status != NEW.status THEN
        CASE 
            WHEN OLD.status = 'DRAFT' AND NEW.status NOT IN ('PENDING_APPROVAL', 'CANCELLED') THEN
                RAISE EXCEPTION 'Invalid status transition from DRAFT to %', NEW.status;
            WHEN OLD.status = 'PENDING_APPROVAL' AND NEW.status NOT IN ('APPROVED', 'DRAFT', 'CANCELLED') THEN
                RAISE EXCEPTION 'Invalid status transition from PENDING_APPROVAL to %', NEW.status;
            WHEN OLD.status = 'APPROVED' AND NEW.status NOT IN ('POSTED', 'CANCELLED') THEN
                RAISE EXCEPTION 'Invalid status transition from APPROVED to %', NEW.status;
            WHEN OLD.status = 'POSTED' AND NEW.status != 'POSTED' THEN
                RAISE EXCEPTION 'Cannot change status of posted journal entry';
            WHEN OLD.status = 'CANCELLED' THEN
                RAISE EXCEPTION 'Cannot change status of cancelled journal entry';
        END CASE;
    END IF;
    
    IF NEW.status = 'POSTED' THEN
        NEW.is_posted := true;
        IF NEW.posted_at IS NULL THEN
            NEW.posted_at := NOW();
        END IF;
    ELSIF NEW.status != 'POSTED' AND OLD.status = 'POSTED' THEN
        RAISE EXCEPTION 'Cannot unpost a journal entry';
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_journal_entry_status
    BEFORE UPDATE ON journal_entries
    FOR EACH ROW
    EXECUTE FUNCTION validate_status_transition();