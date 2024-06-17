-- Load the "uuid-ossp" extension, which provides functions to generate UUIDs
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create a function named 'set_updated_at' to set the 'updated_at' column to the current timestamp
CREATE OR REPLACE FUNCTION set_updated_at ()
    RETURNS TRIGGER
    AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$
LANGUAGE plpgsql;

-- Create a function named 'trigger_updated_at' that creates a trigger for updating the 'updated_at' column
-- After you apply this function to a table, every time before any row of the table is updated, the updated_at column is updated
CREATE OR REPLACE FUNCTION trigger_updated_at (tablename regclass)
    RETURNS void
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at
        BEFORE UPDATE
        ON %s
        FOR EACH ROW
        WHEN (OLD is distinct from NEW)
    EXECUTE FUNCTION set_updated_at();', tablename);
END;
$$
LANGUAGE plpgsql;

-- Create a case-insensitive collation using ICU (International Components for Unicode)
CREATE COLLATION case_insensitive (
    provider = icu,
    locale = 'und-u-ks-level2',
    deterministic = FALSE
);

