ALTER TABLE users ADD COLUMN membership_code varchar NOT NULL DEFAULT 'starter',
ADD COLUMN membership_is_annual boolean NOT NULL DEFAULT FALSE,
ADD COLUMN membership_updated_at timestamptz NULL;
