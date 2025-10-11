CREATE TABLE sessions (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    token citext NOT NULL,
    mango3_auth_token citext NOT NULL,
    mango3_auth_expires_at timestamptz NOT NULL,
    mango3_auth_refreshed_at timestamptz NULL,
    finished_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_sessions PRIMARY KEY (id),
    CONSTRAINT fkey_sessions_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_sessions_on_token ON sessions USING btree (token);
CREATE UNIQUE INDEX index_sessions_on_mango3_auth_token ON sessions USING btree (mango3_auth_token);

SELECT manage_updated_at('sessions');
SELECT manage_versions('sessions');

ALTER TABLE users ADD COLUMN mango3_user_id uuid NULL, ADD COLUMN initials varchar NOT NULL DEFAULT '',
DROP COLUMN encrypted_password;

CREATE UNIQUE INDEX index_users_on_mango3_user_id ON users USING btree (mango3_user_id);

DROP TABLE access_tokens;
