use sqlx::PgPool;
use uuid::Uuid;

/// Permission service for checking user permissions
pub struct PermissionService;

impl PermissionService {
    /// Check if a user has a specific permission
    pub async fn has_permission(
        pool: &PgPool,
        user_id: Uuid,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users u
                JOIN role_permissions rp ON u.role_id = rp.role_id
                JOIN permissions p ON rp.permission_id = p.id
                WHERE u.id = $1 AND p.name = $2 AND u.status = 'active'
            )
            "#,
        )
        .bind(user_id)
        .bind(permission)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Check if a user has any of the specified permissions
    pub async fn has_any_permission(
        pool: &PgPool,
        user_id: Uuid,
        permissions: &[&str],
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users u
                JOIN role_permissions rp ON u.role_id = rp.role_id
                JOIN permissions p ON rp.permission_id = p.id
                WHERE u.id = $1 AND p.name = ANY($2) AND u.status = 'active'
            )
            "#,
        )
        .bind(user_id)
        .bind(permissions)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Check if a user has all of the specified permissions
    pub async fn has_all_permissions(
        pool: &PgPool,
        user_id: Uuid,
        permissions: &[&str],
    ) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT p.name)
            FROM users u
            JOIN role_permissions rp ON u.role_id = rp.role_id
            JOIN permissions p ON rp.permission_id = p.id
            WHERE u.id = $1 AND p.name = ANY($2) AND u.status = 'active'
            "#,
        )
        .bind(user_id)
        .bind(permissions)
        .fetch_one(pool)
        .await?;

        Ok(count == permissions.len() as i64)
    }

    /// Get all permissions for a user
    pub async fn get_user_permissions(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT p.name
            FROM users u
            JOIN role_permissions rp ON u.role_id = rp.role_id
            JOIN permissions p ON rp.permission_id = p.id
            WHERE u.id = $1 AND u.status = 'active'
            ORDER BY p.name
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// Check if a user has a specific role
    pub async fn has_role(
        pool: &PgPool,
        user_id: Uuid,
        role_name: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users u
                JOIN roles r ON u.role_id = r.id
                WHERE u.id = $1 AND r.name = $2 AND u.status = 'active'
            )
            "#,
        )
        .bind(user_id)
        .bind(role_name)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Check if user is admin
    pub async fn is_admin(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
        Self::has_role(pool, user_id, "admin").await
    }
}
