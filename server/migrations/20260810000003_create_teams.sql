CREATE TYPE team_role AS ENUM ('observer', 'responder', 'manager');

CREATE TABLE teams (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    invitation_code VARCHAR(32) UNIQUE,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE team_members (
    id UUID PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role team_role NOT NULL DEFAULT 'observer',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT team_members_unique_membership UNIQUE (team_id, user_id)
);

CREATE INDEX team_members_team_id_idx ON team_members(team_id);
CREATE INDEX team_members_user_id_idx ON team_members(user_id);

CREATE UNIQUE INDEX team_members_one_manager_per_team
    ON team_members(team_id)
    WHERE role = 'manager';

CREATE TABLE team_bans (
    id UUID PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    banned_by UUID REFERENCES users(id) ON DELETE SET NULL,
    until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT team_bans_unique_ban UNIQUE (team_id, user_id)
);

CREATE INDEX team_bans_team_id_idx ON team_bans(team_id);
CREATE INDEX team_bans_until_idx ON team_bans(until);
