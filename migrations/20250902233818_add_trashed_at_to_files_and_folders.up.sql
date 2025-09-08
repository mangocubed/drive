ALTER TABLE files ADD COLUMN trashed_at timestamptz NULL, DROP CONSTRAINT fkey_files_to_parent_folders,
ADD CONSTRAINT fkey_files_to_parent_folders FOREIGN KEY (parent_folder_id) REFERENCES folders (id);

ALTER TABLE folders ADD COLUMN trashed_at timestamptz NULL, DROP CONSTRAINT fkey_folders_to_parent_folders,
ADD CONSTRAINT fkey_folders_to_parent_folders FOREIGN KEY (parent_folder_id) REFERENCES folders (id);
