CREATE TABLE plans (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    name citext NOT NULL,
    description text NOT NULL,
    quota_gib smallint NOT NULL,
    monthly_price_cents smallint NOT NULL,
    yearly_price_cents smallint NOT NULL,
    polar_monthly_product_id uuid NOT NULL,
    polar_yearly_product_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_plans PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_plans_on_name ON plans USING btree (name);
CREATE UNIQUE INDEX index_plans_on_polar_monthly_product_id ON plans USING btree (polar_monthly_product_id);
CREATE UNIQUE INDEX index_plans_on_polar_yearly_product_id ON plans USING btree (polar_yearly_product_id);

ALTER TABLE users
DROP COLUMN total_space_bytes,
ADD COLUMN plan_id uuid NULL,
ADD COLUMN polar_subscription_id uuid NULL,
ADD COLUMN plan_expires_at timestamptz NULL,
ADD CONSTRAINT fkey_users_to_plans FOREIGN KEY (plan_id) REFERENCES plans (id);

CREATE UNIQUE INDEX index_users_on_polar_subscription_id ON users USING btree (polar_subscription_id);
