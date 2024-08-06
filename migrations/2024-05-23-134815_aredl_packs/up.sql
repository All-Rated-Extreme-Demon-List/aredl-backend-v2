CREATE TABLE aredl_pack_tiers (
    id uuid DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    color VARCHAR NOT NULL,
    placement int NOT NULL DEFAULT 0,
    PRIMARY KEY(id)
);

CREATE TABLE aredl_packs (
    id uuid DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    tier uuid NOT NULL REFERENCES aredl_pack_tiers(id) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY(id)
);

CREATE TABLE aredl_pack_levels (
    pack_id uuid REFERENCES aredl_packs(id) ON DELETE CASCADE ON UPDATE CASCADE,
    level_id uuid REFERENCES aredl_levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY(pack_id, level_id)
);

CREATE VIEW aredl_packs_points AS
    SELECT p.*, ROUND(SUM(l.points) * 0.5)::INTEGER AS points
    FROM aredl_packs p
    JOIN aredl_pack_levels pl ON p.id = pl.pack_id
    JOIN aredl_levels l ON l.id = pl.level_id
    GROUP BY p.id;