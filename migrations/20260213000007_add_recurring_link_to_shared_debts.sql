-- Add recurring_debt_id to shared_debts table to link generated debts to their recurring parent
ALTER TABLE shared_debts ADD COLUMN recurring_debt_id INTEGER REFERENCES recurring_debts(id) ON DELETE SET NULL;

-- Create index for efficient lookups
CREATE INDEX idx_shared_debts_recurring_debt_id ON shared_debts(recurring_debt_id);
