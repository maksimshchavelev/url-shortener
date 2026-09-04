ALTER TABLE links
    ADD COLUMN clicks       BIGINT      DEFAULT 0         NOT NULL,
    ADD COLUMN clicks_limit BIGINT      DEFAULT NULL,
    ADD COLUMN created_at   TIMESTAMPTZ DEFAULT NOW()     NOT NULL,
    ADD COLUMN expires_at   TIMESTAMPTZ DEFAULT NULL,
    ADD COLUMN creator_ip   VARCHAR(45) DEFAULT '0.0.0.0' NOT NULL -- maximum IPv6 length
