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