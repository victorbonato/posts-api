CREATE TABLE "post" (
    post_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc (),
    user_id uuid NOT NULL REFERENCES "user" (user_id) ON DELETE CASCADE,
    title text NOT NULL,
    content text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

SELECT
    trigger_updated_at ('"post"');

