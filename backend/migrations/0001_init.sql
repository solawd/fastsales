CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    price_cents INTEGER NOT NULL,
    stock INTEGER NOT NULL,
    product_type TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS product_details (
    id TEXT PRIMARY KEY NOT NULL,
    product_id TEXT NOT NULL,
    detail_name TEXT NOT NULL,
    detail_value TEXT NOT NULL,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    middle_name TEXT,
    mobile_number TEXT NOT NULL,
    date_of_birth TEXT NOT NULL,
    email TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sales (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    date_of_sale TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    discount INTEGER NOT NULL,
    total_cents INTEGER NOT NULL,
    total_resolved INTEGER NOT NULL,
    note TEXT
);

CREATE TABLE IF NOT EXISTS staff (
    id TEXT PRIMARY KEY NOT NULL,
    staff_id TEXT UNIQUE NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    mobile_number TEXT NOT NULL,
    photo_link TEXT NOT NULL,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL
);

INSERT OR IGNORE INTO staff (
    id,
    staff_id,
    first_name,
    last_name,
    mobile_number,
    photo_link,
    username,
    password_hash
) VALUES (
    '550e8400-e29b-41d4-a716-446655440000',
    'staff-0001',
    'Default',
    'Admin',
    '0000000000',
    '',
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$oCHxIpzb3utwH9Su3ATNJg$vRV/UtfIN3f6iKEHrgKgQhoJXM5CoTIFXf9wD9uvrSE'
);
