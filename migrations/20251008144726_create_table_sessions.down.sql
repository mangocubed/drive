CREATE TABLE access_tokens (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    token citext NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_user_access_tokens PRIMARY KEY (id)
);

ALTER TABLE users DROP COLUMN mango3_user_id, DROP COLUMN initials,
ADD COLUMN encrypted_password varchar NOT NULL DEFAULT '';

DROP TABLE sessions;
