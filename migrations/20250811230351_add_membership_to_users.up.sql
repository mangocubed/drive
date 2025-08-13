ALTER TABLE users ADD COLUMN membership_code varchar NOT NULL DEFAULT 'free',
ADD COLUMN has_annual_billing boolean NOT NULL DEFAULT FALSE;
