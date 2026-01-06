use crate::models::{
    tag::{CreateTag, Tag, TagSummary, TagWithCount, UpdateTag},
    DocumentListItem, DocumentOwner,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TagRepository;

impl TagRepository {
    /// 创建标签
    pub async fn create(
        pool: &PgPool,
        tag_data: &CreateTag,
        owner_id: Uuid,
    ) -> Result<Tag, sqlx::Error> {
        sqlx::query_as::<_, Tag>(
            "INSERT INTO tags (id, name, color, owner_id) VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(Uuid::new_v4())
        .bind(&tag_data.name)
        .bind(&tag_data.color)
        .bind(owner_id)
        .fetch_one(pool)
        .await
    }

    /// 根据 ID 查找标签
    pub async fn find_by_id(pool: &PgPool, tag_id: Uuid) -> Result<Option<Tag>, sqlx::Error> {
        sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = $1")
            .bind(tag_id)
            .fetch_optional(pool)
            .await
    }

    /// 列出用户的所有标签（带文档计数）
    pub async fn list_by_owner(
        pool: &PgPool,
        owner_id: Uuid,
    ) -> Result<Vec<TagWithCount>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct TagRow {
            id: Uuid,
            name: String,
            color: String,
            owner_id: Uuid,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            document_count: i64,
        }

        let rows = sqlx::query_as::<_, TagRow>(
            "SELECT t.id, t.name, t.color, t.owner_id, t.created_at, t.updated_at,
             COUNT(dt.document_id) as document_count
             FROM tags t
             LEFT JOIN document_tags dt ON t.id = dt.tag_id
             WHERE t.owner_id = $1
             GROUP BY t.id
             ORDER BY t.name"
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|row| TagWithCount {
            tag: Tag {
                id: row.id,
                name: row.name,
                color: row.color,
                owner_id: row.owner_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            document_count: row.document_count,
        }).collect())
    }

    /// 更新标签
    pub async fn update(
        pool: &PgPool,
        tag_id: Uuid,
        update_data: &UpdateTag,
    ) -> Result<Tag, sqlx::Error> {
        let current = Self::find_by_id(pool, tag_id).await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let name = update_data.name.as_ref().unwrap_or(&current.name);
        let color = update_data.color.as_ref().unwrap_or(&current.color);

        sqlx::query_as::<_, Tag>(
            "UPDATE tags SET name = $1, color = $2, updated_at = NOW() WHERE id = $3 RETURNING *"
        )
        .bind(name)
        .bind(color)
        .bind(tag_id)
        .fetch_one(pool)
        .await
    }

    /// 删除标签
    pub async fn delete(pool: &PgPool, tag_id: Uuid) -> Result<(), sqlx::Error> {
        crate::repository::crud::delete("tags", pool, tag_id).await
    }

    /// 检查用户是否是标签所有者
    pub async fn is_owner(
        pool: &PgPool,
        tag_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        Ok(sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM tags WHERE id = $1 AND owner_id = $2)"
        )
        .bind(tag_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?)
    }

    /// 为文档添加标签
    pub async fn add_to_document(
        pool: &PgPool,
        document_id: Uuid,
        tag_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM document_tags WHERE document_id = $1 AND tag_id = $2)"
        )
        .bind(document_id)
        .bind(tag_id)
        .fetch_one(pool)
        .await?;

        if !exists {
            sqlx::query("INSERT INTO document_tags (document_id, tag_id) VALUES ($1, $2)")
                .bind(document_id)
                .bind(tag_id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// 从文档移除标签
    pub async fn remove_from_document(
        pool: &PgPool,
        document_id: Uuid,
        tag_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM document_tags WHERE document_id = $1 AND tag_id = $2")
            .bind(document_id)
            .bind(tag_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 获取文档的所有标签
    pub async fn get_document_tags(
        pool: &PgPool,
        document_id: Uuid,
    ) -> Result<Vec<TagSummary>, sqlx::Error> {
        sqlx::query_as::<_, TagSummary>(
            "SELECT t.id, t.name, t.color FROM tags t
             INNER JOIN document_tags dt ON t.id = dt.tag_id
             WHERE dt.document_id = $1
             ORDER BY t.name"
        )
        .bind(document_id)
        .fetch_all(pool)
        .await
    }

    /// 批量设置文档标签
    pub async fn set_document_tags(
        pool: &PgPool,
        document_id: Uuid,
        tag_ids: &[Uuid],
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        sqlx::query("DELETE FROM document_tags WHERE document_id = $1")
            .bind(document_id)
            .execute(&mut *tx)
            .await?;
        for &tag_id in tag_ids {
            sqlx::query("INSERT INTO document_tags (document_id, tag_id) VALUES ($1, $2)")
                .bind(document_id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// 按标签筛选文档
    pub async fn get_documents_by_tags(
        pool: &PgPool,
        user_id: Uuid,
        tag_ids: &[Uuid],
        match_all: bool,
    ) -> Result<Vec<DocumentListItem>, sqlx::Error> {
        if tag_ids.is_empty() {
            return Ok(vec![]);
        }

        #[derive(sqlx::FromRow)]
        struct DocumentRow {
            id: Uuid,
            title: String,
            is_public: bool,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            owner_id: Uuid,
            owner_nickname: String,
            owner_email: String,
        }

        let query = if match_all {
            format!(
                "SELECT DISTINCT d.id, d.title, d.is_public, d.created_at, d.updated_at,
                 u.id as owner_id, u.nickname as owner_nickname, u.email as owner_email
                 FROM documents d
                 INNER JOIN users u ON d.owner_id = u.id
                 WHERE d.owner_id = $1
                 AND d.id IN (
                     SELECT document_id FROM document_tags
                     WHERE tag_id = ANY($2)
                     GROUP BY document_id
                     HAVING COUNT(DISTINCT tag_id) = $3
                 )
                 ORDER BY d.updated_at DESC"
            )
        } else {
            "SELECT DISTINCT d.id, d.title, d.is_public, d.created_at, d.updated_at,
             u.id as owner_id, u.nickname as owner_nickname, u.email as owner_email
             FROM documents d
             INNER JOIN users u ON d.owner_id = u.id
             INNER JOIN document_tags dt ON d.id = dt.document_id
             WHERE d.owner_id = $1 AND dt.tag_id = ANY($2)
             ORDER BY d.updated_at DESC"
                .to_string()
        };

        let doc_rows: Vec<DocumentRow> = if match_all {
            sqlx::query_as(&query).bind(user_id).bind(tag_ids).bind(tag_ids.len() as i64).fetch_all(pool).await?
        } else {
            sqlx::query_as(&query).bind(user_id).bind(tag_ids).fetch_all(pool).await?
        };

        Ok(doc_rows.into_iter().map(|row| DocumentListItem {
            id: row.id,
            title: row.title,
            is_public: row.is_public,
            created_at: row.created_at,
            updated_at: row.updated_at,
            owner: DocumentOwner {
                id: row.owner_id,
                nickname: row.owner_nickname,
                email: row.owner_email,
            },
        }).collect())
    }
}
