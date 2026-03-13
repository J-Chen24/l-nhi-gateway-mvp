-- Create wallets table
CREATE TABLE wallets (
    id UUID PRIMARY KEY,
    owner_name TEXT NOT NULL,
    balance BIGINT NOT NULL CHECK (balance >= 0)
);

-- Create audit_logs table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    wallet_id UUID,
    action TEXT NOT NULL,
    amount BIGINT,
    status TEXT NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
