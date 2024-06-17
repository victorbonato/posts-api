CREATE TABLE "user" (
    user_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc (),
    username text COLLATE "case_insensitive" UNIQUE NOT NULL,
    password_hash text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

-- Call the 'trigger_updated_at' function to create a trigger for the "user" table
-- This trigger will automatically update the 'updated_at' column to the current timestamp
-- whenever a row in the "user" table is updated
SELECT
    trigger_updated_at ('"user"');

