use crate::models::Role;
use sqlx::PgPool;
use uuid::Uuid;

pub struct RoleRepository;

impl RoleRepository {
    /// Find role by ID
    pub async fn find_by_id(pool: &PgPool, role_id: Uuid) -> Result<Option<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1")
            .bind(role_id)
            .fetch_optional(pool)
            .await
    }

    /// Find role by name
    pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
    }

    /// List all roles
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>("SELECT * FROM roles ORDER BY created_at")
            .fetch_all(pool)
            .await
    }

    /// Create a new role (non-system)
    pub async fn create(
        pool: &PgPool,
        name: &str,
        description: Option<&str>,
    ) -> Result<Role, sqlx::Error> {
        let role_id = Uuid::new_v4();

        sqlx::query_as::<_, Role>(
            r#"
            INSERT INTO roles (id, name, description, is_system)
            VALUES ($1, $2, $3, false)
            RETURNING *
            "#,
        )
        .bind(role_id)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
    }

    /// Update role
    pub async fn update(
        pool: &PgPool,
        role_id: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<Role, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            r#"
            UPDATE roles
            SET name = $1, description = $2
            WHERE id = $3 AND is_system = false
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(role_id)
        .fetch_one(pool)
        .await
    }

    /// Delete role (only non-system roles)
    pub async fn delete(pool: &PgPool, role_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM roles WHERE id = $1 AND is_system = false")
            .bind(role_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Get all permissions for a role
    pub async fn get_permissions(
        pool: &PgPool,
        role_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT p.name
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            "#,
            role_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.name).collect())
    }

    /// Add permission to role
    pub async fn add_permission(
        pool: &PgPool,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(role_id)
        .bind(permission_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Remove permission from role
    pub async fn remove_permission(
        pool: &PgPool,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2")
            .bind(role_id)
            .bind(permission_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
