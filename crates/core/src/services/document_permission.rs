use entangle_db::{models::CollaboratorPermission, DocumentRepository, PermissionRepository};
use sqlx::PgPool;
use uuid::Uuid;

/// 文档权限服务
pub struct DocumentPermissionService;

impl DocumentPermissionService {
    /// 检查用户是否可以读取文档
    pub async fn can_read(
        pool: &PgPool,
        user_id: Uuid,
        doc_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // 1. 检查是否是文档所有者
        if let Some(doc) = DocumentRepository::find_by_id(pool, doc_id).await? {
            if doc.owner_id == user_id {
                return Ok(true);
            }

            // 2. 检查文档是否公开
            if doc.is_public {
                return Ok(true);
            }
        } else {
            return Ok(false);
        }

        // 3. 检查是否是协作者
        let permission = DocumentRepository::get_user_permission(pool, doc_id, user_id).await?;
        Ok(permission.is_some())
    }

    /// 检查用户是否可以编辑文档
    pub async fn can_write(
        pool: &PgPool,
        user_id: Uuid,
        doc_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // 1. 检查是否是文档所有者
        if let Some(doc) = DocumentRepository::find_by_id(pool, doc_id).await? {
            if doc.owner_id == user_id {
                return Ok(true);
            }
        } else {
            return Ok(false);
        }

        // 2. 检查是否有写入或管理权限
        let permission = DocumentRepository::get_user_permission(pool, doc_id, user_id).await?;
        Ok(matches!(
            permission,
            Some(CollaboratorPermission::Write) | Some(CollaboratorPermission::Admin)
        ))
    }

    /// 检查用户是否可以管理文档（添加/删除协作者、删除文档）
    pub async fn can_manage(
        pool: &PgPool,
        user_id: Uuid,
        doc_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // 1. 检查是否是文档所有者
        if let Some(doc) = DocumentRepository::find_by_id(pool, doc_id).await? {
            if doc.owner_id == user_id {
                return Ok(true);
            }
        } else {
            return Ok(false);
        }

        // 2. 检查是否有管理权限
        let permission = DocumentRepository::get_user_permission(pool, doc_id, user_id).await?;
        Ok(matches!(permission, Some(CollaboratorPermission::Admin)))
    }

    /// 检查用户是否可以删除文档（仅所有者）
    pub async fn can_delete(
        pool: &PgPool,
        user_id: Uuid,
        doc_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        if let Some(doc) = DocumentRepository::find_by_id(pool, doc_id).await? {
            Ok(doc.owner_id == user_id)
        } else {
            Ok(false)
        }
    }

    /// 检查用户是否是文档所有者
    pub async fn is_owner(
        pool: &PgPool,
        user_id: Uuid,
        doc_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        Self::can_delete(pool, user_id, doc_id).await
    }

    /// 检查用户是否有 document:* 权限（基于 RBAC）
    pub async fn has_document_permission(
        pool: &PgPool,
        user_id: Uuid,
        action: &str,
    ) -> Result<bool, sqlx::Error> {
        let permission_name = format!("document:{}", action);
        PermissionRepository::user_has_permission(pool, user_id, &permission_name).await
    }
}
