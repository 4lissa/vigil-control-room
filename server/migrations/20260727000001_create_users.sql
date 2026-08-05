CREATE TABLE users (
    id            UUID PRIMARY KEY,
    username      VARCHAR(50) NOT NULL UNIQUE,
    email         VARCHAR(254) NOT NULL UNIQUE,
    password_hash TEXT,
    github_id     TEXT UNIQUE,
    language      TEXT NOT NULL DEFAULT 'en',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT has_auth_method CHECK (
        password_hash IS NOT NULL OR github_id IS NOT NULL
    )
);
