-- 文档版本表
CREATE TABLE document_versions (
    id UUID PRIMARY KEY,
    doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    crdt_state BYTEA,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    description VARCHAR(500),  -- 版本描述/备注
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- 确保同一文档的版本号唯一
    UNIQUE(doc_id, version_number)
);

CREATE INDEX idx_versions_doc ON document_versions(doc_id);
CREATE INDEX idx_versions_created_by ON document_versions(created_by);
CREATE INDEX idx_versions_created_at ON document_versions(created_at);
