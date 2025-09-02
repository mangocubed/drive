CREATE TABLE access_tokens (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    token citext NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_user_access_tokens PRIMARY KEY (id),
    CONSTRAINT fkey_user_access_tokens_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_user_access_tokens_on_token ON access_tokens USING btree (token);

SELECT manage_updated_at('access_tokens');
SELECT manage_versions('access_tokens');

DROP TABLE user_sessions;
