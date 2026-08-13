CREATE TABLE knowledge_link_index (
    source_kind TEXT NOT NULL CHECK (source_kind IN ('problem', 'knowledge')),
    source_id TEXT NOT NULL,
    target_ref TEXT NOT NULL,
    target_knowledge_node_id TEXT REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
    resolution TEXT NOT NULL CHECK (resolution IN ('resolved', 'unresolved', 'ambiguous', 'non_knowledge_target')),
    PRIMARY KEY (source_kind, source_id, target_ref)
);

CREATE INDEX knowledge_link_index_by_target
ON knowledge_link_index(target_knowledge_node_id);

UPDATE app_metadata
SET schema_generation = 13
WHERE singleton = 1;
