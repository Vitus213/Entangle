use crate::models::Permission;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PermissionRepository;

impl PermissionRepository {
    /// Find permission by ID
    pub async fn find_by_id(
        pool: &PgPool,
        permission_id: Uuid,
    ) -> Result<Option<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>("SELECT * FROM permissions WHERE id = $1")
            .bind(permission_id)
            .fetch_optional(pool)
            .await
    }

    /// Find permission by name
    pub async fn find_by_name(
        pool: &PgPool,
        name: &str,
    ) -> Result<Option<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>("SELECT * FROM permissions WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
    }

    /// List all permissions
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>("SELECT * FROM permissions ORDER BY resource, action")
            .fetch_all(pool)
            .await
    }

    /// List permissions by resource
    pub async fn list_by_resource(
        pool: &PgPool,
        resource: &str,
    ) -> Result<Vec<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>("SELECT * FROM permissions WHERE resource = $1 ORDER BY action")
            .bind(resource)
            .fetch_all(pool)
            .await
    }

    /// Create a new permission
    pub async fn create(
        pool: &PgPool,
        name: &str,
        resource: &str,
        action: &str,
        description: Option<&str>,
    ) -> Result<Permission, sqlx::Error> {
        let permission_id = Uuid::new_v4();

        sqlx::query_as::<_, Permission>(
            r#"
            INSERT INTO permissions (id, name, resource, action, description)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(permission_id)
        .bind(name)
        .bind(resource)
        .bind(action)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    /// Update permission
    pub async fn update(
        pool: &PgPool,
        permission_id: Uuid,
        name: &str,
        resource: &str,
        action: &str,
        description: Option<&str>,
    ) -> Result<Permission, sqlx::Error> {
        sqlx::query_as::<_, Permission>(
            r#"
            UPDATE permissions
            SET name = $1, resource = $2, action = $3, description = $4
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(resource)
        .bind(action)
        .bind(description)
        .bind(permission_id)
        .fetch_one(pool)
        .await
    }

    /// Delete permission
    pub async fn delete(pool: &PgPool, permission_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM permissions WHERE id = $1")
            .bind(permission_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Check if user has specific permission
    pub async fn user_has_permission(
        pool: &PgPool,
        user_id: Uuid,
        permission_name: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users u
                JOIN role_permissions rp ON u.role_id = rp.role_id
                JOIN permissions p ON rp.permission_id = p.id
                WHERE u.id = $1 AND p.name = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(permission_name)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Get all permissions for a user
    pub async fn get_user_permissions(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT p.name
            FROM users u
            JOIN role_permissions rp ON u.role_id = rp.role_id
            JOIN permissions p ON rp.permission_id = p.id
            WHERE u.id = $1
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.name).collect())
    }
}
