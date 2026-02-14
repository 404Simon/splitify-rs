-- Create shared_debts table
CREATE TABLE IF NOT EXISTS shared_debts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    created_by INTEGER NOT NULL,
    name TEXT NOT NULL,
    amount TEXT NOT NULL,  -- Stored as TEXT to maintain DECIMAL precision (rust_decimal serializes to string)
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);

-- Create shared_debt_user pivot table
CREATE TABLE IF NOT EXISTS shared_debt_user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shared_debt_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (shared_debt_id) REFERENCES shared_debts(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(shared_debt_id, user_id)
);

-- Indexes for performance
CREATE INDEX idx_shared_debts_group_id ON shared_debts(group_id);
CREATE INDEX idx_shared_debts_created_by ON shared_debts(created_by);
CREATE INDEX idx_shared_debt_user_shared_debt_id ON shared_debt_user(shared_debt_id);
CREATE INDEX idx_shared_debt_user_user_id ON shared_debt_user(user_id);
