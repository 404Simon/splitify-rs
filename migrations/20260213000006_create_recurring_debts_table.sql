-- Create recurring_debts table
CREATE TABLE recurring_debts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    created_by INTEGER NOT NULL,
    name TEXT NOT NULL,
    amount TEXT NOT NULL,  -- Stored as TEXT for rust_decimal precision
    frequency TEXT NOT NULL CHECK(frequency IN ('daily', 'weekly', 'monthly', 'yearly')),
    start_date DATE NOT NULL,
    end_date DATE,  -- NULL = infinite
    next_generation_date DATE NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes for efficient queries
CREATE INDEX idx_recurring_debts_group_id ON recurring_debts(group_id);
CREATE INDEX idx_recurring_debts_created_by ON recurring_debts(created_by);
CREATE INDEX idx_recurring_debts_next_generation_date ON recurring_debts(next_generation_date);
CREATE INDEX idx_recurring_debts_is_active ON recurring_debts(is_active);

-- Create recurring_debt_user pivot table
CREATE TABLE recurring_debt_user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recurring_debt_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (recurring_debt_id) REFERENCES recurring_debts(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(recurring_debt_id, user_id)
);

-- Create indexes for pivot table
CREATE INDEX idx_recurring_debt_user_recurring_debt_id ON recurring_debt_user(recurring_debt_id);
CREATE INDEX idx_recurring_debt_user_user_id ON recurring_debt_user(user_id);
