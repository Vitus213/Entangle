use crate::models::notification::{
    CreateNotification, Notification, NotificationResponse, NotificationSender,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct NotificationRepository;

impl NotificationRepository {
    /// 创建通知
    pub async fn create(
        pool: &PgPool,
        input: CreateNotification,
    ) -> Result<Notification, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (id, user_id, type, title, content, resource_type, resource_id, sender_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.user_id)
        .bind(input.notification_type.to_string())
        .bind(&input.title)
        .bind(&input.content)
        .bind(input.resource_type.map(|rt| rt.to_string()))
        .bind(input.resource_id)
        .bind(input.sender_id)
        .fetch_one(pool)
        .await
    }

    /// 批量创建通知（同一内容发给多个用户）
    pub async fn create_batch(
        pool: &PgPool,
        user_ids: &[Uuid],
        notification_type: &str,
        title: &str,
        content: Option<&str>,
        resource_type: Option<&str>,
        resource_id: Option<Uuid>,
        sender_id: Option<Uuid>,
    ) -> Result<Vec<Notification>, sqlx::Error> {
        let mut notifications = Vec::new();
        for user_id in user_ids {
            let id = Uuid::new_v4();
            let notification = sqlx::query_as::<_, Notification>(
                r#"
                INSERT INTO notifications (id, user_id, type, title, content, resource_type, resource_id, sender_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING *
                "#,
            )
            .bind(id)
            .bind(user_id)
            .bind(notification_type)
            .bind(title)
            .bind(content)
            .bind(resource_type)
            .bind(resource_id)
            .bind(sender_id)
            .fetch_one(pool)
            .await?;
            notifications.push(notification);
        }
        Ok(notifications)
    }

    /// 获取用户的通知列表
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NotificationResponse>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, Option<String>, Option<String>, Option<Uuid>, bool, chrono::DateTime<chrono::Utc>, Option<Uuid>, Option<String>, Option<String>)>(
            r#"
            SELECT
                n.id,
                n.type,
                n.title,
                n.content,
                n.resource_type,
                n.resource_id,
                n.is_read,
                n.created_at,
                u.id as sender_id,
                u.nickname as sender_nickname,
                u.avatar_url as sender_avatar_url
            FROM notifications n
            LEFT JOIN users u ON n.sender_id = u.id
            WHERE n.user_id = $1
            ORDER BY n.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| NotificationResponse {
                id: row.0,
                notification_type: row.1,
                title: row.2,
                content: row.3,
                resource_type: row.4,
                resource_id: row.5,
                is_read: row.6,
                created_at: row.7,
                sender: row.8.map(|id| NotificationSender {
                    id,
                    nickname: row.9.unwrap_or_default(),
                    avatar_url: row.10,
                }),
            })
            .collect())
    }

    /// 获取未读通知数量
    pub async fn count_unread(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = FALSE",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    /// 标记单个通知为已读
    pub async fn mark_read(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE notifications SET is_read = TRUE WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 标记所有通知为已读
    pub async fn mark_all_read(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result =
            sqlx::query("UPDATE notifications SET is_read = TRUE WHERE user_id = $1 AND is_read = FALSE")
                .bind(user_id)
                .execute(pool)
                .await?;
        Ok(result.rows_affected())
    }

    /// 删除通知
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 删除用户的所有已读通知
    pub async fn delete_read(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result =
            sqlx::query("DELETE FROM notifications WHERE user_id = $1 AND is_read = TRUE")
                .bind(user_id)
                .execute(pool)
                .await?;
        Ok(result.rows_affected())
    }

    /// 清理过期通知（超过 30 天的已读通知）
    pub async fn cleanup_old(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM notifications WHERE is_read = TRUE AND created_at < NOW() - INTERVAL '30 days'",
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}
