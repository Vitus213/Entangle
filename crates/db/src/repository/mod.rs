use sqlx::PgPool;
use uuid::Uuid;

pub mod user;
pub mod role;
pub mod permission;
pub mod document;
pub mod folder;
pub mod tag;
pub mod comment;
pub mod notification;
pub mod task;
pub mod version;

pub use user::UserRepository;
pub use role::RoleRepository;
pub use permission::PermissionRepository;
pub use document::DocumentRepository;
pub use folder::FolderRepository;
pub use tag::TagRepository;
pub use comment::CommentRepository;
pub use notification::NotificationRepository;
pub use task::TaskRepository;
pub use version::VersionRepository;

/// 简单的 CRUD 辅助函数
pub mod crud {
    use super::*;

    /// 通用删除函数
    pub async fn delete(table: &str, pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(&format!("DELETE FROM {} WHERE id = $1", table))
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
