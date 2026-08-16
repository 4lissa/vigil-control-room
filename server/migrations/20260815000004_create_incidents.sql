CREATE TYPE incident_state AS ENUM ('open', 'acknowledged', 'escalated', 'resolved');
CREATE TYPE severity AS ENUM ('low', 'medium', 'high', 'critical');

CREATE TABLE incidents (
    id UUID PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    state incident_state NOT NULL DEFAULT 'open',
    severity severity NOT NULL,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX incidents_team_id_idx ON incidents(team_id);
CREATE INDEX incidents_state_idx ON incidents(state);
CREATE INDEX incidents_assigned_to_idx ON incidents(assigned_to);

CREATE TABLE timeline_entries (
    id UUID PRIMARY KEY,
    incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    author_id UUID REFERENCES users(id) ON DELETE SET NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    edited_at TIMESTAMPTZ
);

CREATE INDEX timeline_entries_incident_id_idx ON timeline_entries(incident_id);
