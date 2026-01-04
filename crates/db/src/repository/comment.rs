use crate::models::comment::{
    Comment, CommentListItem, CommentPosition, CommentResponse, CommentUser, CreateComment,
    UpdateComment,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct CommentRepository;

impl CommentRepository {
    /// 创建评论
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        input: CreateComment,
    ) -> Result<Comment, sqlx::Error> {
        let position_json = input
            .position
            .map(|p| serde_json::to_value(p).unwrap_or_default());
        let id = Uuid::new_v4();

        sqlx::query_as::<_, Comment>(
            r#"
            INSERT INTO comments (id, doc_id, user_id, parent_id, content, position)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.doc_id)
        .bind(user_id)
        .bind(input.parent_id)
        .bind(&input.content)
        .bind(position_json)
        .fetch_one(pool)
        .await
    }

    /// 根据 ID 获取评论
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Comment>, sqlx::Error> {
        sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 获取文档的所有评论（不包括回复）
    pub async fn find_by_document(
        pool: &PgPool,
        doc_id: Uuid,
    ) -> Result<Vec<CommentListItem>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (
            Uuid, Uuid, Option<Uuid>, String,
            Option<sqlx::types::Json<CommentPosition>>, bool,
            chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>,
            Uuid, String, Option<String>, i64
        )>(
            r#"
            SELECT
                c.id,
                c.doc_id,
                c.parent_id,
                c.content,
                c.position,
                c.is_resolved,
                c.created_at,
                c.updated_at,
                u.id as user_id,
                u.nickname as user_nickname,
                u.avatar_url as user_avatar_url,
                COALESCE(reply_count.count, 0)::bigint as reply_count
            FROM comments c
            JOIN users u ON c.user_id = u.id
            LEFT JOIN (
                SELECT parent_id, COUNT(*) as count
                FROM comments
                WHERE parent_id IS NOT NULL
                GROUP BY parent_id
            ) reply_count ON reply_count.parent_id = c.id
            WHERE c.doc_id = $1 AND c.parent_id IS NULL
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(doc_id)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| CommentListItem {
                id: row.0,
                doc_id: row.1,
                parent_id: row.2,
                content: row.3,
                position: row.4.map(|p| p.0),
                is_resolved: row.5,
                created_at: row.6,
                updated_at: row.7,
                user: CommentUser {
                    id: row.8,
                    nickname: row.9,
                    avatar_url: row.10,
                },
                reply_count: row.11,
            })
            .collect())
    }

    /// 获取评论的回复
    pub async fn find_replies(
        pool: &PgPool,
        parent_id: Uuid,
    ) -> Result<Vec<CommentResponse>, sqlx::Error> {
        // 简化版：只获取一层回复
        let rows = sqlx::query_as::<_, (Uuid, Uuid, Option<Uuid>, String, Option<sqlx::types::Json<CommentPosition>>, bool, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, Uuid, String, Option<String>)>(
            r#"
            SELECT
                c.id,
                c.doc_id,
                c.parent_id,
                c.content,
                c.position,
                c.is_resolved,
                c.created_at,
                c.updated_at,
                u.id,
                u.nickname,
                u.avatar_url
            FROM comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.parent_id = $1
            ORDER BY c.created_at ASC
            "#,
        )
        .bind(parent_id)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| CommentResponse {
                id: row.0,
                doc_id: row.1,
                parent_id: row.2,
                content: row.3,
                position: row.4.map(|p| p.0),
                is_resolved: row.5,
                created_at: row.6,
                updated_at: row.7,
                user: CommentUser {
                    id: row.8,
                    nickname: row.9,
                    avatar_url: row.10,
                },
                replies: vec![], // 不递归获取
            })
            .collect())
    }

    /// 获取评论详情（包含回复）
    pub async fn find_with_replies(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<CommentResponse>, sqlx::Error> {
        let row = sqlx::query_as::<_, (Uuid, Uuid, Option<Uuid>, String, Option<sqlx::types::Json<CommentPosition>>, bool, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, Uuid, String, Option<String>)>(
            r#"
            SELECT
                c.id,
                c.doc_id,
                c.parent_id,
                c.content,
                c.position,
                c.is_resolved,
                c.created_at,
                c.updated_at,
                u.id,
                u.nickname,
                u.avatar_url
            FROM comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let replies = Self::find_replies(pool, id).await?;
                Ok(Some(CommentResponse {
                    id: row.0,
                    doc_id: row.1,
                    parent_id: row.2,
                    content: row.3,
                    position: row.4.map(|p| p.0),
                    is_resolved: row.5,
                    created_at: row.6,
                    updated_at: row.7,
                    user: CommentUser {
                        id: row.8,
                        nickname: row.9,
                        avatar_url: row.10,
                    },
                    replies,
                }))
            }
            None => Ok(None),
        }
    }

    /// 更新评论
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateComment,
    ) -> Result<Comment, sqlx::Error> {
        sqlx::query_as::<_, Comment>(
            r#"
            UPDATE comments
            SET content = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(&input.content)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// 标记评论为已解决
    pub async fn resolve(pool: &PgPool, id: Uuid) -> Result<Comment, sqlx::Error> {
        sqlx::query_as::<_, Comment>(
            r#"
            UPDATE comments
            SET is_resolved = TRUE, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// 取消解决状态
    pub async fn unresolve(pool: &PgPool, id: Uuid) -> Result<Comment, sqlx::Error> {
        sqlx::query_as::<_, Comment>(
            r#"
            UPDATE comments
            SET is_resolved = FALSE, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    /// 删除评论
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM comments WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 检查用户是否是评论的作者
    pub async fn is_author(pool: &PgPool, comment_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM comments WHERE id = $1 AND user_id = $2",
        )
        .bind(comment_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result > 0)
    }

    /// 获取评论数量
    pub async fn count_by_document(pool: &PgPool, doc_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comments WHERE doc_id = $1")
            .bind(doc_id)
            .fetch_one(pool)
            .await
    }
}
