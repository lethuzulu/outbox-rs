
ALTER TABLE outbox_messages
    ADD COLUMN locked_until TIMESTAMPTZ;