CREATE TYPE reaction_emoji AS ENUM ('+1', '-1', 'eyes', 'warning', 'check', 'fire');

CREATE TABLE timeline_reactions (
    id UUID PRIMARY KEY,
    entry_id UUID NOT NULL REFERENCES timeline_entries(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji reaction_emoji NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT timeline_reactions_unique_reaction UNIQUE (entry_id, user_id, emoji)
);

CREATE INDEX timeline_reactions_entry_id_idx ON timeline_reactions(entry_id);
