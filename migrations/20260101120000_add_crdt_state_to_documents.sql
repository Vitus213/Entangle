-- Add CRDT state column to documents table
ALTER TABLE documents ADD COLUMN crdt_state BYTEA;

-- Add index for documents with CRDT state
CREATE INDEX idx_documents_has_crdt ON documents(id) WHERE crdt_state IS NOT NULL;
