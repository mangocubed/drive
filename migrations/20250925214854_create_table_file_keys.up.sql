CREATE TABLE file_keys (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    file_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_file_keys PRIMARY KEY (id),
    CONSTRAINT fkey_file_keys_to_files FOREIGN KEY (file_id) REFERENCES files (id) ON DELETE CASCADE
);

SELECT manage_updated_at('file_keys');
SELECT manage_versions('file_keys');
