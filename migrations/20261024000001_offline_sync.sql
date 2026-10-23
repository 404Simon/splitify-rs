-- Offline-first sync support: idempotent mutations and timestamp normalisation.

-- Store the first successful response for a mutation so outbox retries can be
-- replayed instead of re-executed. Rows older than 7 days are purged.
CREATE TABLE IF NOT EXISTS api_idempotency_keys (
    token_hash TEXT NOT NULL,
    key TEXT NOT NULL,
    method TEXT NOT NULL,
    path TEXT NOT NULL,
    status INTEGER NOT NULL,
    body TEXT NOT NULL,
    content_type TEXT,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (token_hash, key, method, path)
);

CREATE INDEX IF NOT EXISTS idx_api_idempotency_keys_created_at
    ON api_idempotency_keys(created_at);

-- Normalise shopping-list timestamps written as RFC3339 text to the SQLite
-- space-separated format used by CURRENT_TIMESTAMP.
UPDATE shopping_lists
SET updated_at = strftime('%Y-%m-%d %H:%M:%S', updated_at)
WHERE updated_at LIKE '%T%';

UPDATE shopping_list_items
SET updated_at = strftime('%Y-%m-%d %H:%M:%S', updated_at)
WHERE updated_at LIKE '%T%';
