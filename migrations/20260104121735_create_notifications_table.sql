-- 通知表
CREATE TABLE notifications (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL,  -- comment, mention, task, share, system
    title VARCHAR(255) NOT NULL,
    content TEXT,
    resource_type VARCHAR(50),  -- document, comment, task
    resource_id UUID,
    sender_id UUID REFERENCES users(id) ON DELETE SET NULL,  -- 发送者
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_notifications_user ON notifications(user_id);
CREATE INDEX idx_notifications_unread ON notifications(user_id, is_read) WHERE is_read = FALSE;
CREATE INDEX idx_notifications_created ON notifications(created_at);
