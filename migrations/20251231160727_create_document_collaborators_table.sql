-- Create document_collaborators table
CREATE TABLE document_collaborators (
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(20) NOT NULL CHECK (permission IN ('read', 'write', 'admin')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (document_id, user_id)
);

-- Create indexes
CREATE INDEX idx_document_collaborators_user ON document_collaborators(user_id);
CREATE INDEX idx_document_collaborators_document ON document_collaborators(document_id);
