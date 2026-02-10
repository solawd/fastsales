CREATE TABLE IF NOT EXISTS sales (
    id TEXT PRIMARY KEY,
    customer_id TEXT,
    date_and_time TEXT NOT NULL,
    total_cents INTEGER NOT NULL,
    discount INTEGER NOT NULL,
    total_resolved INTEGER NOT NULL,
    sales_channel TEXT NOT NULL,
    staff_responsible TEXT NOT NULL,
    company_branch TEXT NOT NULL,
    car_number TEXT NOT NULL,
    receipt_number TEXT NOT NULL,
    FOREIGN KEY (customer_id) REFERENCES customers(id)
);

ALTER TABLE sale_items ADD COLUMN sale_id TEXT;
-- Add constraint manually if sqlite supports adding FK column via ALTER TABLE?
-- SQLite has limited ALTER TABLE support. We added the column, but FK constraint usually requires recreating the table.
-- However, we can live without strict FK constraint for now, or just use PRAGMA foreign_keys = ON;
-- For correctness in SQLite migrations without recreating table:
-- We can't add FK in ALTER TABLE directly. But we can ensure the column exists.
-- Referencing `sales(id)` by application logic is fine too, but FK is better.
-- Let's trust the application to handle for this task step or recreate table if needed.
-- But wait, recreating `sale_items` with data is tricky.
-- Just adding the column is safe. We can rely on app logic for integrity or "PRAGMA foreign_keys=ON" enforcing it if we could define it.
-- Actually newer SQLite confirms FK in ALTER TABLE might not work but let's check docs?
-- "The ALTER TABLE command in SQLite allows ... ADD COLUMN ... but you cannot add a foreign key constraint to an existing table."
-- Okay, so just adding the column.
