use crate::models::{
    AddCollaborator, CollaboratorPermission, CreateDocument, Document, DocumentCollaborator,
    DocumentListItem, DocumentOwner, DocumentResponse, UpdateDocument,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct DocumentRepository;

impl DocumentRepository {
    /// 创建文档
    pub async fn create(
        pool: &PgPool,
        owner_id: Uuid,
        doc: &CreateDocument,
    ) -> Result<Document, sqlx::Error> {
        let doc_id = Uuid::new_v4();

        sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (id, title, content, owner_id, is_public)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(doc_id)
        .bind(&doc.title)
        .bind(&doc.content)
        .bind(owner_id)
        .bind(doc.is_public)
        .fetch_one(pool)
        .await
    }

    /// 根据 ID 查找文档
    pub async fn find_by_id(pool: &PgPool, doc_id: Uuid) -> Result<Option<Document>, sqlx::Error> {
        sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(doc_id)
            .fetch_optional(pool)
            .await
    }

    /// 获取文档的 CRDT 状态
    pub async fn get_crdt_state(pool: &PgPool, doc_id: Uuid) -> Result<Option<Vec<u8>>, sqlx::Error> {
        let result = sqlx::query_scalar::<_, Option<Vec<u8>>>(
            "SELECT crdt_state FROM documents WHERE id = $1"
        )
        .bind(doc_id)
        .fetch_optional(pool)
        .await?;

        Ok(result.flatten())
    }

    /// 保存文档的 CRDT 状态
    pub async fn save_crdt_state(
        pool: &PgPool,
        doc_id: Uuid,
        crdt_state: &[u8],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE documents SET crdt_state = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(crdt_state)
        .bind(doc_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 同时保存 CRDT 状态和文本内容（用于同步）
    pub async fn save_crdt_state_with_content(
        pool: &PgPool,
        doc_id: Uuid,
        crdt_state: &[u8],
        content: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE documents SET crdt_state = $1, content = $2, updated_at = NOW() WHERE id = $3"
        )
        .bind(crdt_state)
        .bind(content)
        .bind(doc_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 获取文档详情（包含作者信息）
    pub async fn get_detail(
        pool: &PgPool,
        doc_id: Uuid,
    ) -> Result<Option<DocumentResponse>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT
                d.id, d.title, d.content, d.is_public, d.created_at, d.updated_at,
                u.id as owner_id, u.nickname as owner_nickname, u.email as owner_email
            FROM documents d
            JOIN users u ON d.owner_id = u.id
            WHERE d.id = $1
            "#,
            doc_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|row| DocumentResponse {
            id: row.id,
            title: row.title,
            content: row.content,
            owner: DocumentOwner {
                id: row.owner_id,
                nickname: row.owner_nickname,
                email: row.owner_email,
            },
            is_public: row.is_public,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }

    /// 更新文档
    pub async fn update(
        pool: &PgPool,
        doc_id: Uuid,
        update: &UpdateDocument,
    ) -> Result<Document, sqlx::Error> {
        // 构建动态更新语句
        let mut query = String::from("UPDATE documents SET updated_at = NOW()");
        let mut param_count = 1;

        if update.title.is_some() {
            param_count += 1;
            query.push_str(&format!(", title = ${}", param_count));
        }
        if update.content.is_some() {
            param_count += 1;
            query.push_str(&format!(", content = ${}", param_count));
        }
        if update.is_public.is_some() {
            param_count += 1;
            query.push_str(&format!(", is_public = ${}", param_count));
        }

        query.push_str(" WHERE id = $1 RETURNING *");

        let mut q = sqlx::query_as::<_, Document>(&query).bind(doc_id);

        if let Some(ref title) = update.title {
            q = q.bind(title);
        }
        if let Some(ref content) = update.content {
            q = q.bind(content);
        }
        if let Some(is_public) = update.is_public {
            q = q.bind(is_public);
        }

        q.fetch_one(pool).await
    }

    /// 删除文档
    pub async fn delete(pool: &PgPool, doc_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(doc_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 列出用户的文档
    pub async fn list_by_owner(
        pool: &PgPool,
        owner_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DocumentListItem>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                d.id, d.title, d.is_public, d.created_at, d.updated_at,
                u.id as owner_id, u.nickname as owner_nickname, u.email as owner_email
            FROM documents d
            JOIN users u ON d.owner_id = u.id
            WHERE d.owner_id = $1
            ORDER BY d.updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
            owner_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| DocumentListItem {
                id: row.id,
                title: row.title,
                owner: DocumentOwner {
                    id: row.owner_id,
                    nickname: row.owner_nickname,
                    email: row.owner_email,
                },
                is_public: row.is_public,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    /// 列出公开文档
    pub async fn list_public(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DocumentListItem>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                d.id, d.title, d.is_public, d.created_at, d.updated_at,
                u.id as owner_id, u.nickname as owner_nickname, u.email as owner_email
            FROM documents d
            JOIN users u ON d.owner_id = u.id
            WHERE d.is_public = TRUE
            ORDER BY d.updated_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| DocumentListItem {
                id: row.id,
                title: row.title,
                owner: DocumentOwner {
                    id: row.owner_id,
                    nickname: row.owner_nickname,
                    email: row.owner_email,
                },
                is_public: row.is_public,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    /// 添加协作者
    pub async fn add_collaborator(
        pool: &PgPool,
        doc_id: Uuid,
        collab: &AddCollaborator,
    ) -> Result<DocumentCollaborator, sqlx::Error> {
        // openGauss 不支持 ON CONFLICT，先删除再插入
        sqlx::query("DELETE FROM document_collaborators WHERE document_id = $1 AND user_id = $2")
            .bind(doc_id)
            .bind(collab.user_id)
            .execute(pool)
            .await?;

        sqlx::query_as::<_, DocumentCollaborator>(
            r#"
            INSERT INTO document_collaborators (document_id, user_id, permission)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(doc_id)
        .bind(collab.user_id)
        .bind(collab.permission.to_string())
        .fetch_one(pool)
        .await
    }

    /// 移除协作者
    pub async fn remove_collaborator(
        pool: &PgPool,
        doc_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM document_collaborators WHERE document_id = $1 AND user_id = $2")
            .bind(doc_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 获取文档的协作者列表
    pub async fn list_collaborators(
        pool: &PgPool,
        doc_id: Uuid,
    ) -> Result<Vec<DocumentCollaborator>, sqlx::Error> {
        sqlx::query_as::<_, DocumentCollaborator>(
            "SELECT * FROM document_collaborators WHERE document_id = $1",
        )
        .bind(doc_id)
        .fetch_all(pool)
        .await
    }

    /// 获取用户对文档的权限
    pub async fn get_user_permission(
        pool: &PgPool,
        doc_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<CollaboratorPermission>, sqlx::Error> {
        let result = sqlx::query_scalar::<_, String>(
            "SELECT permission FROM document_collaborators WHERE document_id = $1 AND user_id = $2",
        )
        .bind(doc_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(result.and_then(|perm| match perm.as_str() {
            "read" => Some(CollaboratorPermission::Read),
            "write" => Some(CollaboratorPermission::Write),
            "admin" => Some(CollaboratorPermission::Admin),
            _ => None,
        }))
    }

    /// 列出用户可访问的文档（自己的 + 协作的）
    pub async fn list_accessible(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DocumentListItem>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT
                d.id, d.title, d.is_public, d.created_at, d.updated_at,
                u.id as owner_id, u.nickname as owner_nickname, u.email as owner_email
            FROM documents d
            JOIN users u ON d.owner_id = u.id
            LEFT JOIN document_collaborators dc ON d.id = dc.document_id
            WHERE d.owner_id = $1 OR dc.user_id = $1
            ORDER BY d.updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| DocumentListItem {
                id: row.id,
                title: row.title,
                owner: DocumentOwner {
                    id: row.owner_id,
                    nickname: row.owner_nickname,
                    email: row.owner_email,
                },
                is_public: row.is_public,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }

    /// 搜索文档（支持标题搜索和筛选）
    pub async fn search(
        pool: &PgPool,
        user_id: Uuid,
        query: Option<&str>,
        folder_id: Option<Uuid>,
        tag_id: Option<Uuid>,
        is_public: Option<bool>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DocumentListItem>, sqlx::Error> {
        let mut conditions = vec![
            "(d.owner_id = $1 OR EXISTS (
                SELECT 1 FROM document_collaborators dc
                WHERE dc.document_id = d.id AND dc.user_id = $1
            ) OR d.is_public = true)".to_string()
        ];

        let mut param_count = 1;

        // 标题搜索（使用 ILIKE 实现模糊匹配）
        let title_param = if let Some(q) = query {
            param_count += 1;
            conditions.push(format!("d.title ILIKE ${}", param_count));
            Some(format!("%{}%", q))
        } else {
            None
        };

        // 文件夹筛选
        let folder_param = if let Some(fid) = folder_id {
            param_count += 1;
            conditions.push(format!("d.folder_id = ${}", param_count));
            Some(fid)
        } else {
            None
        };

        // 标签筛选
        let tag_param = if let Some(tid) = tag_id {
            param_count += 1;
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM document_tags dt WHERE dt.document_id = d.id AND dt.tag_id = ${})",
                param_count
            ));
            Some(tid)
        } else {
            None
        };

        // 公开状态筛选
        let public_param = if let Some(pub_flag) = is_public {
            param_count += 1;
            conditions.push(format!("d.is_public = ${}", param_count));
            Some(pub_flag)
        } else {
            None
        };

        let where_clause = conditions.join(" AND ");

        param_count += 1;
        let limit_param = param_count;
        param_count += 1;
        let offset_param = param_count;

        let sql = format!(
            r#"
            SELECT DISTINCT
                d.id, d.title, d.is_public, d.created_at, d.updated_at,
                u.id as owner_id, u.nickname as owner_nickname, u.email as owner_email
            FROM documents d
            JOIN users u ON d.owner_id = u.id
            WHERE {}
            ORDER BY d.updated_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause, limit_param, offset_param
        );

        let mut query_builder = sqlx::query(&sql).bind(user_id);

        if let Some(ref title) = title_param {
            query_builder = query_builder.bind(title);
        }
        if let Some(fid) = folder_param {
            query_builder = query_builder.bind(fid);
        }
        if let Some(tid) = tag_param {
            query_builder = query_builder.bind(tid);
        }
        if let Some(pub_flag) = public_param {
            query_builder = query_builder.bind(pub_flag);
        }

        query_builder = query_builder.bind(limit).bind(offset);

        let rows = query_builder
            .map(|row: sqlx::postgres::PgRow| {
                use sqlx::Row;
                DocumentListItem {
                    id: row.get("id"),
                    title: row.get("title"),
                    owner: DocumentOwner {
                        id: row.get("owner_id"),
                        nickname: row.get("owner_nickname"),
                        email: row.get("owner_email"),
                    },
                    is_public: row.get("is_public"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .fetch_all(pool)
            .await?;

        Ok(rows)
    }

    /// 复制文档
    pub async fn duplicate(
        pool: &PgPool,
        doc_id: Uuid,
        owner_id: Uuid,
        new_title: Option<String>,
    ) -> Result<Document, sqlx::Error> {
        // 获取原文档
        let original = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(doc_id)
            .fetch_one(pool)
            .await?;

        // 创建新文档
        let new_id = Uuid::new_v4();
        let title = new_title.unwrap_or_else(|| format!("{} (副本)", original.title));

        sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (id, title, content, owner_id, is_public)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(new_id)
        .bind(title)
        .bind(&original.content)
        .bind(owner_id)
        .bind(false) // 复制的文档默认为私有
        .fetch_one(pool)
        .await
    }
}
