-- Create folders table for hierarchical document organization
CREATE TABLE folders (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    parent_id UUID,
    owner_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 确保文件夹名称不为空
    CONSTRAINT folders_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT folders_parent_fk FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE,
    CONSTRAINT folders_owner_fk FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 性能优化索引
CREATE INDEX idx_folders_parent ON folders(parent_id) TABLESPACE pg_default;
CREATE INDEX idx_folders_owner ON folders(owner_id) TABLESPACE pg_default;
CREATE INDEX idx_folders_owner_parent ON folders(owner_id, parent_id) TABLESPACE pg_default;

-- 为 documents 表添加 folder_id 列
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'entangle'
        AND table_name = 'documents'
        AND column_name = 'folder_id'
    ) THEN
        ALTER TABLE documents ADD COLUMN folder_id UUID;
        ALTER TABLE documents ADD CONSTRAINT documents_folder_fk
            FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL;
        CREATE INDEX idx_documents_folder ON documents(folder_id) TABLESPACE pg_default;
    END IF;
END $$;
