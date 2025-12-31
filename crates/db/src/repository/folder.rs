use crate::models::{
    Folder, CreateFolder, UpdateFolder, FolderResponse,
    FolderTree, FolderContents, FolderInfo, FolderSummary,
    OwnerInfo, DocumentListItem,
};
use sqlx::PgPool;
use uuid::Uuid;

/// 文件夹仓库
pub struct FolderRepository;

impl FolderRepository {
    /// 创建文件夹
    pub async fn create(
        pool: &PgPool,
        folder: &CreateFolder,
        owner_id: Uuid,
    ) -> Result<Folder, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, Folder>(
            r#"
            INSERT INTO folders (id, name, parent_id, owner_id)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&folder.name)
        .bind(folder.parent_id)
        .bind(owner_id)
        .fetch_one(pool)
        .await
    }

    /// 根据 ID 获取文件夹
    pub async fn find_by_id(
        pool: &PgPool,
        folder_id: Uuid,
    ) -> Result<Option<Folder>, sqlx::Error> {
        sqlx::query_as::<_, Folder>(
            "SELECT * FROM folders WHERE id = $1"
        )
        .bind(folder_id)
        .fetch_optional(pool)
        .await
    }

    /// 获取文件夹详情（包含所有者信息）
    pub async fn get_detail(
        pool: &PgPool,
        folder_id: Uuid,
    ) -> Result<Option<FolderResponse>, sqlx::Error> {
        // 分别查询文件夹和所有者信息
        let folder = Self::find_by_id(pool, folder_id).await?;

        if let Some(f) = folder {
            let owner: OwnerInfo = sqlx::query_as(
                "SELECT id, nickname, email FROM users WHERE id = $1"
            )
            .bind(f.owner_id)
            .fetch_one(pool)
            .await?;

            Ok(Some(FolderResponse {
                id: f.id,
                name: f.name,
                parent_id: f.parent_id,
                owner,
                created_at: f.created_at,
                updated_at: f.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// 更新文件夹
    pub async fn update(
        pool: &PgPool,
        folder_id: Uuid,
        update: &UpdateFolder,
    ) -> Result<Folder, sqlx::Error> {
        // 先获取当前文件夹
        let current = Self::find_by_id(pool, folder_id).await?
            .ok_or(sqlx::Error::RowNotFound)?;

        // 使用提供的值或保持当前值
        let name = update.name.as_ref().unwrap_or(&current.name);
        let parent_id = if update.parent_id.is_some() {
            update.parent_id
        } else {
            current.parent_id
        };

        sqlx::query_as::<_, Folder>(
            r#"
            UPDATE folders
            SET name = $1, parent_id = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#
        )
        .bind(name)
        .bind(parent_id)
        .bind(folder_id)
        .fetch_one(pool)
        .await
    }

    /// 删除文件夹
    pub async fn delete(
        pool: &PgPool,
        folder_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM folders WHERE id = $1")
            .bind(folder_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 获取用户的文件夹树
    pub async fn get_tree(
        pool: &PgPool,
        owner_id: Uuid,
    ) -> Result<Vec<FolderTree>, sqlx::Error> {
        // 获取所有文件夹
        let folders = sqlx::query_as::<_, Folder>(
            "SELECT * FROM folders WHERE owner_id = $1 ORDER BY name"
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await?;

        // 获取每个文件夹的文档数量
        let doc_counts: Vec<(Uuid, i64)> = sqlx::query_as(
            r#"
            SELECT folder_id, COUNT(*) as count
            FROM documents
            WHERE folder_id IS NOT NULL AND owner_id = $1
            GROUP BY folder_id
            "#
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await?;

        let count_map: std::collections::HashMap<Uuid, i64> =
            doc_counts.into_iter().collect();

        // 构建树形结构
        Ok(build_tree(folders, &count_map, None))
    }

    /// 获取文件夹内容
    pub async fn get_contents(
        pool: &PgPool,
        folder_id: Uuid,
    ) -> Result<FolderContents, sqlx::Error> {
        // 获取文件夹信息和路径
        let path = Self::get_path(pool, folder_id).await?;
        let folder_opt = Self::find_by_id(pool, folder_id).await?;

        let folder = folder_opt.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let folder_info = FolderInfo {
            id: folder.id,
            name: folder.name,
            path,
        };

        // 获取子文件夹
        let subfolders: Vec<FolderSummary> = sqlx::query_as(
            r#"
            SELECT
                f.id,
                f.name,
                COUNT(d.id) as document_count
            FROM folders f
            LEFT JOIN documents d ON d.folder_id = f.id
            WHERE f.parent_id = $1
            GROUP BY f.id, f.name
            ORDER BY f.name
            "#
        )
        .bind(folder_id)
        .fetch_all(pool)
        .await?;

        // 获取文档 - 使用临时结构
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

        let doc_rows: Vec<DocumentRow> = sqlx::query_as(
            r#"
            SELECT
                d.id, d.title, d.is_public, d.created_at, d.updated_at,
                u.id as owner_id,
                u.nickname as owner_nickname,
                u.email as owner_email
            FROM documents d
            INNER JOIN users u ON d.owner_id = u.id
            WHERE d.folder_id = $1
            ORDER BY d.updated_at DESC
            "#
        )
        .bind(folder_id)
        .fetch_all(pool)
        .await?;

        // 转换为 DocumentListItem
        let documents: Vec<DocumentListItem> = doc_rows
            .into_iter()
            .map(|row| DocumentListItem {
                id: row.id,
                title: row.title,
                is_public: row.is_public,
                created_at: row.created_at,
                updated_at: row.updated_at,
                owner: crate::models::DocumentOwner {
                    id: row.owner_id,
                    nickname: row.owner_nickname,
                    email: row.owner_email,
                },
            })
            .collect();

        Ok(FolderContents {
            folder: folder_info,
            subfolders,
            documents,
        })
    }

    /// 获取文件夹路径
    pub async fn get_path(
        pool: &PgPool,
        folder_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        let path: Vec<String> = match sqlx::query_scalar(
            r#"
            WITH RECURSIVE folder_path AS (
                SELECT id, name, parent_id, ARRAY[name::TEXT] as path
                FROM folders
                WHERE id = $1

                UNION ALL

                SELECT f.id, f.name, f.parent_id, f.name::TEXT || fp.path
                FROM folders f
                INNER JOIN folder_path fp ON f.id = fp.parent_id
            )
            SELECT path FROM folder_path WHERE parent_id IS NULL
            "#
        )
        .bind(folder_id)
        .fetch_optional(pool)
        .await?
        {
            Some(p) => p,
            None => {
                sqlx::query_scalar("SELECT ARRAY[name::TEXT] FROM folders WHERE id = $1")
                    .bind(folder_id)
                    .fetch_one(pool)
                    .await?
            }
        };

        Ok(path)
    }

    /// 检查用户是否是文件夹所有者
    pub async fn is_owner(
        pool: &PgPool,
        folder_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT owner_id = $1 FROM folders WHERE id = $2"
        )
        .bind(user_id)
        .bind(folder_id)
        .fetch_optional(pool)
        .await?;

        Ok(result.unwrap_or(false))
    }
}

/// 构建文件夹树
fn build_tree(
    folders: Vec<Folder>,
    doc_counts: &std::collections::HashMap<Uuid, i64>,
    parent_id: Option<Uuid>,
) -> Vec<FolderTree> {
    folders
        .iter()
        .filter(|f| f.parent_id == parent_id)
        .map(|f| {
            let children = build_tree(folders.clone(), doc_counts, Some(f.id));
            FolderTree {
                folder: f.clone(),
                children,
                document_count: *doc_counts.get(&f.id).unwrap_or(&0),
            }
        })
        .collect()
}
