-- Update total_resolved: boolean (0/1) -> amount (cents)
-- If total_resolved was 1 (true), it means it was fully resolved, so set it to total_cents.
UPDATE sales SET total_resolved = total_cents WHERE total_resolved = 1;

-- Update date_of_sale: simple date (YYYY-MM-DD) -> ISO 8601 timestamp (YYYY-MM-DDTHH:MM:SSZ)
-- Only update if format is currently strictly YYYY-MM-DD (length 10).
UPDATE sales SET date_of_sale = date_of_sale || 'T00:00:00Z' WHERE length(date_of_sale) = 10;
