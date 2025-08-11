CREATE TABLE files (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    parent_folder_id uuid NULL,
    name citext NOT NULL,
    visibility file_visibility NOT NULL DEFAULT 'private',
    media_type varchar NOT NULL,
    byte_size bigint NOT NULL,
    md5_checksum varchar NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_files PRIMARY KEY (id),
    CONSTRAINT fkey_files_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT fkey_files_to_parent_folders FOREIGN KEY (parent_folder_id) REFERENCES folders (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_files_on_user_id_parent_folder_id_name ON files USING btree (user_id, parent_folder_id, name);

SELECT manage_updated_at('files');
SELECT manage_versions('files');
