CREATE TABLE IF NOT EXISTS customer_details (
    id TEXT PRIMARY KEY NOT NULL,
    customer_id TEXT NOT NULL,
    detail_name TEXT NOT NULL,
    detail_value TEXT NOT NULL,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);
