CREATE TYPE file_visibility AS ENUM ('private', 'followers', 'users', 'public');

CREATE TABLE folders (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    parent_folder_id uuid NULL,
    name citext NOT NULL,
    visibility file_visibility NOT NULL DEFAULT 'private',
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_folders PRIMARY KEY (id),
    CONSTRAINT fkey_folders_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT fkey_folders_to_parent_folders FOREIGN KEY (parent_folder_id) REFERENCES folders (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_folders_on_user_id_parent_folder_id_name ON folders (user_id, parent_folder_id, name);

SELECT manage_updated_at('folders');
SELECT manage_versions('folders');
