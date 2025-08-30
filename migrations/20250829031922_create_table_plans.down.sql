ALTER TABLE users
ADD COLUMN total_space_bytes bigint NOT NULL DEFAULT 1073741824,
DROP COLUMN total_space_gib,
DROP COLUMN plan_id,
DROP COLUMN polar_subscription_id,
DROP COLUMN polar_subscription_expires_at;

DROP TABLE plans;
