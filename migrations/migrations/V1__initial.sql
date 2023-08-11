CREATE TABLE IF NOT EXISTS hanger (
    id         varchar NOT NULL,
    deadline    timestamptz NOT NULL DEFAULT now(),
    readied_at timestamptz,
    readied_by uuid,
    message_headers jsonb,
    message_key varchar,
    message_value json,
    PRIMARY KEY ("id")
)