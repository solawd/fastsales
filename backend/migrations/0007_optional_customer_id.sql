-- Make customer_id optional in sale_items
-- We need to recreate the table because SQLite has limited ALTER TABLE support for modifying column constraints.

CREATE TABLE IF NOT EXISTS sale_items_new (
    id TEXT PRIMARY KEY,
    sale_id TEXT,
    product_id TEXT NOT NULL,
    customer_id TEXT, -- Now nullable
    date_of_sale TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    discount INTEGER NOT NULL,
    total_cents INTEGER NOT NULL,
    total_resolved INTEGER NOT NULL,
    note TEXT,
    product_name TEXT,
    price_per_item INTEGER
);

-- Copy data
INSERT INTO sale_items_new (id, sale_id, product_id, customer_id, date_of_sale, quantity, discount, total_cents, total_resolved, note, product_name, price_per_item)
SELECT id, sale_id, product_id, customer_id, date_of_sale, quantity, discount, total_cents, total_resolved, note, product_name, price_per_item
FROM sale_items;

-- Swap tables
DROP TABLE sale_items;
ALTER TABLE sale_items_new RENAME TO sale_items;
