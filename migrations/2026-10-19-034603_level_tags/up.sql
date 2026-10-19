CREATE TYPE tag_category AS ENUM ('Version', 'Length', 'Gamemode', 'Gameplay', 'Other');
ALTER TYPE audit_entity_type ADD VALUE 'Tag';
ALTER TYPE audit_entity_type ADD VALUE 'LevelTag';

CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    description TEXT NOT NULL DEFAULT '',
    category tag_category NOT NULL DEFAULT 'Other',
    UNIQUE (list_id, id),
    UNIQUE (list_id, name)
);

CREATE TABLE level_tags (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    level_id UUID NOT NULL,
    tag_id UUID NOT NULL,
    PRIMARY KEY (list_id, level_id, tag_id),
    FOREIGN KEY (list_id, level_id) REFERENCES levels(list_id, id) ON DELETE CASCADE,
    FOREIGN KEY (list_id, tag_id) REFERENCES tags(list_id, id) ON DELETE CASCADE
);
CREATE INDEX level_tags_tag_idx ON level_tags(list_id, tag_id);

INSERT INTO tags (list_id, name)
SELECT DISTINCT l.list_id, tag
FROM levels l CROSS JOIN LATERAL unnest(l.tags) AS tag
WHERE tag IS NOT NULL AND btrim(tag) <> '';

INSERT INTO level_tags (list_id, level_id, tag_id)
SELECT DISTINCT l.list_id, l.id, t.id
FROM levels l CROSS JOIN LATERAL unnest(l.tags) AS tag
JOIN tags t ON t.list_id = l.list_id AND t.name = tag;

CREATE OR REPLACE VIEW public.badge_level_statistics AS
SELECT r.list_id, r.submitted_by, l.id, l.name,
       COALESCE(completed_state.position, first_placed_state.position) AS position,
       l.position AS current_position, l.level_id, l.two_player, l.publisher_id,
       l.edel_enjoyment, l.nlw_tier, ARRAY(
           SELECT t.name FROM level_tags a
           JOIN tags t ON (t.list_id, t.id) = (a.list_id, a.tag_id)
           WHERE (a.list_id, a.level_id) = (l.list_id, l.id)
           ORDER BY t.name, t.id
       ) AS tags, r.is_verification, r.achieved_at,
       COALESCE(first_record.submitted_by = r.submitted_by, false) AS is_first_victor,
       COALESCE(fastest_record.submitted_by = r.submitted_by, false) AS is_fastest_time,
       r.id AS record_id
FROM records r
JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM position_history_full_view ph
    WHERE ph.list_id = r.list_id AND ph.affected_level = r.level_id AND ph.action_at <= r.achieved_at
    ORDER BY ph.action_at DESC, ph.ord DESC LIMIT 1
) completed_state ON true
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM position_history_full_view ph
    WHERE ph.list_id = r.list_id AND ph.affected_level = r.level_id AND ph.position IS NOT NULL
      AND completed_state.position IS NULL
      AND (completed_state.status IS NULL OR
           (completed_state.status = 'Pending' AND ph.action_at >= r.achieved_at))
    ORDER BY ph.action_at, ph.ord LIMIT 1
) first_placed_state ON true
LEFT JOIN LATERAL (
    SELECT fr.submitted_by FROM records fr
    WHERE fr.list_id = r.list_id AND fr.level_id = r.level_id AND fr.is_verification = false
    ORDER BY fr.achieved_at, fr.created_at, fr.id LIMIT 1
) first_record ON true
LEFT JOIN LATERAL (
    SELECT fr.submitted_by FROM records fr
    WHERE r.list_id = 2 AND fr.list_id = r.list_id
      AND fr.level_id = r.level_id AND fr.is_verification = false
    ORDER BY fr.completion_time, fr.achieved_at, fr.id LIMIT 1
) fastest_record ON true
WHERE completed_state.status IN ('MainList', 'Pending')
   OR (completed_state.status IS NULL AND first_placed_state.position IS NOT NULL);

ALTER TABLE levels DROP COLUMN tags;
