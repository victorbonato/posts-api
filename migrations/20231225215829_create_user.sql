CREATE TABLE "user" (
    user_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc (),
    username text COLLATE "case_insensitive" UNIQUE NOT NULL,
    password_hash text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

SELECT
    trigger_updated_at ('"user"');

