CREATE TABLE rules (
    id UUID PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    trigger_service TEXT NOT NULL,
    trigger_event TEXT NOT NULL,
    trigger_filters JSONB NOT NULL DEFAULT '{}',
    encrypted_webhook_secret BYTEA NOT NULL,
    reaction_type TEXT NOT NULL,
    reaction_payload JSONB NOT NULL DEFAULT '{}',
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX rules_team_id_idx ON rules(team_id);
