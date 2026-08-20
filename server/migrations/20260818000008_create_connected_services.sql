CREATE TABLE connected_services (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    service TEXT NOT NULL,
    encrypted_token BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT connected_services_unique_service UNIQUE (user_id, service)
);
