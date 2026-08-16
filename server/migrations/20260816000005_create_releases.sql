CREATE TYPE release_state AS ENUM ('created', 'in_progress', 'completed', 'cancelled', 'blocked');

CREATE TABLE releases (
    id UUID PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    state release_state NOT NULL DEFAULT 'created',
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX releases_team_id_idx ON releases(team_id);
CREATE INDEX releases_state_idx ON releases(state);

CREATE TABLE release_steps (
    id UUID PRIMARY KEY,
    release_id UUID NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    position INT NOT NULL,
    validated_at TIMESTAMPTZ,
    validated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    CONSTRAINT release_steps_unique_position UNIQUE (release_id, position)
);

CREATE INDEX release_steps_release_id_idx ON release_steps(release_id);

ALTER TABLE incidents ADD COLUMN release_id UUID REFERENCES releases(id) ON DELETE SET NULL;

CREATE INDEX incidents_release_id_idx ON incidents(release_id);
