ALTER TABLE links
    ADD COLUMN clicks       BIGINT      DEFAULT 0,
    ADD COLUMN clicks_limit BIGINT      DEFAULT 0,
    ADD COLUMN created_at   TIMESTAMPTZ DEFAULT NOW(),
    ADD COLUMN expires_at   TIMESTAMPTZ,
    ADD COLUMN created_ip   VARCHAR(45) -- maximum IPv6 length
