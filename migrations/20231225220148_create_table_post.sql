CREATE TABLE "post" (
    post_id bigserial PRIMARY KEY,
    -- The ON DELETE CASCADE clause ensures that if a user is deleted, all their posts will also be deleted
    user_id uuid NOT NULL REFERENCES "user" (user_id) ON DELETE CASCADE,
    title text NOT NULL,
    content text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

-- Call the 'trigger_updated_at' function to create a trigger for the "post" table
-- This trigger will automatically update the 'updated_at' column to the current timestamp
-- whenever a row in the "post" table is updated
SELECT
    trigger_updated_at ('"post"');

