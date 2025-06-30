BEGIN;
-- Fill status column with default value in deployment-safe approach
UPDATE subscriptions
SET status = 'confirmed'
WHERE status IS NULL;
-- Make status column NOT NULL
ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;