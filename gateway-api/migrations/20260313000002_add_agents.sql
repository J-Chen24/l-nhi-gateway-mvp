CREATE TABLE agents (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE wallets
ADD COLUMN agent_id UUID;

INSERT INTO agents (id, name, agent_type)
SELECT
    '00000000-0000-0000-0000-000000000001',
    'Legacy Agent',
    'legacy'
WHERE EXISTS (
    SELECT 1
    FROM wallets
);

UPDATE wallets
SET agent_id = '00000000-0000-0000-0000-000000000001'
WHERE agent_id IS NULL;

ALTER TABLE wallets
ALTER COLUMN agent_id SET NOT NULL;

ALTER TABLE wallets
ADD CONSTRAINT wallets_agent_id_fkey
FOREIGN KEY (agent_id) REFERENCES agents(id);
