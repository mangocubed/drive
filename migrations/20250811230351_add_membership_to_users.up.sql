ALTER TABLE users
ADD COLUMN membership_code varchar NOT NULL DEFAULT 'starter',
ADD COLUMN membership_is_annual boolean NOT NULL DEFAULT FALSE,
ADD COLUMN membership_subscription_id uuid NULL,
ADD COLUMN membership_expires_at timestamptz NULL,
ADD COLUMN membership_updated_at timestamptz NULL;

CREATE UNIQUE INDEX index_users_on_membership_subscription_id ON users(membership_subscription_id);
