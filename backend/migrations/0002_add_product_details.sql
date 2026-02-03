CREATE TABLE product_details (
    id TEXT PRIMARY KEY NOT NULL,
    product_id TEXT NOT NULL,
    detail_name TEXT NOT NULL,
    detail_value TEXT NOT NULL,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);
