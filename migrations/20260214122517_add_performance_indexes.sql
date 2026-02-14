-- Add performance indexes to improve query speed
-- Migration: 20260214122517_add_performance_indexes

-- Users table indexes
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);

-- Groups table indexes
CREATE INDEX IF NOT EXISTS idx_groups_created_by ON groups(created_by);
CREATE INDEX IF NOT EXISTS idx_groups_created_at ON groups(created_at);

-- Group members table indexes
CREATE INDEX IF NOT EXISTS idx_group_members_group_id ON group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_group_members_user_id ON group_members(user_id);
CREATE INDEX IF NOT EXISTS idx_group_members_lookup ON group_members(group_id, user_id);

-- Shared debts table indexes
CREATE INDEX IF NOT EXISTS idx_shared_debts_group_id ON shared_debts(group_id);
CREATE INDEX IF NOT EXISTS idx_shared_debts_created_by ON shared_debts(created_by);
CREATE INDEX IF NOT EXISTS idx_shared_debts_created_at ON shared_debts(created_at);
CREATE INDEX IF NOT EXISTS idx_shared_debts_recurring_debt_id ON shared_debts(recurring_debt_id);

-- Shared debt user pivot table indexes
CREATE INDEX IF NOT EXISTS idx_shared_debt_user_debt_id ON shared_debt_user(shared_debt_id);
CREATE INDEX IF NOT EXISTS idx_shared_debt_user_user_id ON shared_debt_user(user_id);

-- Recurring debts table indexes
CREATE INDEX IF NOT EXISTS idx_recurring_debts_group_id ON recurring_debts(group_id);
CREATE INDEX IF NOT EXISTS idx_recurring_debts_created_by ON recurring_debts(created_by);
CREATE INDEX IF NOT EXISTS idx_recurring_debts_next_generation_date ON recurring_debts(next_generation_date);
CREATE INDEX IF NOT EXISTS idx_recurring_debts_is_active ON recurring_debts(is_active);
CREATE INDEX IF NOT EXISTS idx_recurring_debts_active_due ON recurring_debts(is_active, next_generation_date) 
    WHERE is_active = 1 AND next_generation_date IS NOT NULL;

-- Recurring debt user pivot table indexes
CREATE INDEX IF NOT EXISTS idx_recurring_debt_user_debt_id ON recurring_debt_user(recurring_debt_id);
CREATE INDEX IF NOT EXISTS idx_recurring_debt_user_user_id ON recurring_debt_user(user_id);

-- Transactions table indexes
CREATE INDEX IF NOT EXISTS idx_transactions_group_id ON transactions(group_id);
CREATE INDEX IF NOT EXISTS idx_transactions_payer_id ON transactions(payer_id);
CREATE INDEX IF NOT EXISTS idx_transactions_recipient_id ON transactions(recipient_id);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_group_payer ON transactions(group_id, payer_id);
CREATE INDEX IF NOT EXISTS idx_transactions_group_recipient ON transactions(group_id, recipient_id);

-- Invites table indexes (if exists)
CREATE INDEX IF NOT EXISTS idx_invites_group_id ON invites(group_id);
CREATE INDEX IF NOT EXISTS idx_invites_uuid ON invites(uuid);
CREATE INDEX IF NOT EXISTS idx_invites_created_at ON invites(created_at);
