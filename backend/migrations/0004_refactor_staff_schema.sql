-- Create new table with UUID primary key
CREATE TABLE staff_new (
    id TEXT PRIMARY KEY NOT NULL,
    staff_id TEXT UNIQUE NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    mobile_number TEXT NOT NULL,
    photo_link TEXT NOT NULL,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL
);

-- Copy existing data, generating UUIDs for id
-- Using a standard approximation for UUID v4 in SQLite
INSERT INTO staff_new (id, staff_id, first_name, last_name, mobile_number, photo_link, username, password_hash)
SELECT 
    lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))),2) || '-' || substr('89ab',abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))),2) || '-' || lower(hex(randomblob(6))),
    staff_id, 
    first_name, 
    last_name, 
    mobile_number, 
    photo_link, 
    username, 
    password_hash
FROM staff;

-- Start transaction to swap tables
DROP TABLE staff;
ALTER TABLE staff_new RENAME TO staff;
