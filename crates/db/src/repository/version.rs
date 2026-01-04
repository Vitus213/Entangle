use crate::models::version::{
    DocumentVersion, VersionCreator, VersionListItem, VersionResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct VersionRepository;

impl VersionRepository {
    /// 创建新版本
    pub async fn create(
        pool: &PgPool,
        doc_id: Uuid,
        title: String,
        content: String,
        crdt_state: Option<Vec<u8>>,
        created_by: Uuid,
        description: Option<String>,
    ) -> Result<DocumentVersion, sqlx::Error> {
        let id = Uuid::new_v4();

        // 获取下一个版本号
        let next_version = Self::get_next_version_number(pool, doc_id).await?;

        sqlx::query_as::<_, DocumentVersion>(
            r#"
            INSERT INTO document_versions (id, doc_id, version_number, title, content, crdt_state, created_by, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(doc_id)
        .bind(next_version)
        .bind(&title)
        .bind(&content)
        .bind(&crdt_state)
        .bind(created_by)
        .bind(&description)
        .fetch_one(pool)
        .await
    }

    /// 获取下一个版本号
    async fn get_next_version_number(pool: &PgPool, doc_id: Uuid) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(version_number) FROM document_versions WHERE doc_id = $1",
        )
        .bind(doc_id)
        .fetch_one(pool)
        .await?;

        Ok(result.unwrap_or(0) + 1)
    }

    /// 根据 ID 获取版本
    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<DocumentVersion>, sqlx::Error> {
        sqlx::query_as::<_, DocumentVersion>("SELECT * FROM document_versions WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 获取版本详情（包含创建者信息）
    pub async fn find_with_creator(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<VersionResponse>, sqlx::Error> {
        let row = sqlx::query_as::<_, (
            Uuid, Uuid, i32, String, String, Option<Vec<u8>>, Uuid, Option<String>,
            chrono::DateTime<chrono::Utc>, Uuid, String
        )>(
            r#"
            SELECT
                v.id,
                v.doc_id,
                v.version_number,
                v.title,
                v.content,
                v.crdt_state,
                v.created_by,
                v.description,
                v.created_at,
                u.id as creator_id,
                u.nickname as creator_nickname
            FROM document_versions v
            JOIN users u ON v.created_by = u.id
            WHERE v.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| VersionResponse {
            id: r.0,
            doc_id: r.1,
            version_number: r.2,
            title: r.3,
            content: r.4,
            crdt_state: r.5.map(|s| hex::encode(&s)),
            creator: VersionCreator {
                id: r.9,
                nickname: r.10,
            },
            description: r.7,
            created_at: r.8,
        }))
    }

    /// 获取文档的所有版本列表
    pub async fn find_by_document(
        pool: &PgPool,
        doc_id: Uuid,
    ) -> Result<Vec<VersionListItem>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (
            Uuid, Uuid, i32, String, Option<String>,
            chrono::DateTime<chrono::Utc>, Uuid, String
        )>(
            r#"
            SELECT
                v.id,
                v.doc_id,
                v.version_number,
                v.title,
                v.description,
                v.created_at,
                u.id as creator_id,
                u.nickname as creator_nickname
            FROM document_versions v
            JOIN users u ON v.created_by = u.id
            WHERE v.doc_id = $1
            ORDER BY v.version_number DESC
            "#,
        )
        .bind(doc_id)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| VersionListItem {
                id: r.0,
                doc_id: r.1,
                version_number: r.2,
                title: r.3,
                description: r.4,
                created_at: r.5,
                creator: VersionCreator {
                    id: r.6,
                    nickname: r.7,
                },
            })
            .collect())
    }

    /// 获取文档的最新版本
    pub async fn find_latest(
        pool: &PgPool,
        doc_id: Uuid,
    ) -> Result<Option<DocumentVersion>, sqlx::Error> {
        sqlx::query_as::<_, DocumentVersion>(
            r#"
            SELECT * FROM document_versions
            WHERE doc_id = $1
            ORDER BY version_number DESC
            LIMIT 1
            "#,
        )
        .bind(doc_id)
        .fetch_optional(pool)
        .await
    }

    /// 获取指定版本号的版本
    pub async fn find_by_version_number(
        pool: &PgPool,
        doc_id: Uuid,
        version_number: i32,
    ) -> Result<Option<DocumentVersion>, sqlx::Error> {
        sqlx::query_as::<_, DocumentVersion>(
            "SELECT * FROM document_versions WHERE doc_id = $1 AND version_number = $2",
        )
        .bind(doc_id)
        .bind(version_number)
        .fetch_optional(pool)
        .await
    }

    /// 删除版本
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM document_versions WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 获取文档的版本数量
    pub async fn count_by_document(pool: &PgPool, doc_id: Uuid) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM document_versions WHERE doc_id = $1")
            .bind(doc_id)
            .fetch_one(pool)
            .await
    }

    /// 检查版本是否属于指定文档
    pub async fn belongs_to_document(
        pool: &PgPool,
        version_id: Uuid,
        doc_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM document_versions WHERE id = $1 AND doc_id = $2",
        )
        .bind(version_id)
        .bind(doc_id)
        .fetch_one(pool)
        .await?;

        Ok(result > 0)
    }
}
