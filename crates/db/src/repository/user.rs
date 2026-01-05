use crate::models::{CreateUser, User, UserResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserRepository;

impl UserRepository {
    /// Create a new user
    pub async fn create(
        pool: &PgPool,
        user: &CreateUser,
        password_hash: String,
        default_role_id: Uuid,
    ) -> Result<User, sqlx::Error> {
        let user_id = Uuid::new_v4();

        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, phone, password_hash, nickname, role_id, email_verified, status)
            VALUES ($1, $2, $3, $4, $5, $6, false, 'active')
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(&user.email)
        .bind(&user.phone)
        .bind(password_hash)
        .bind(&user.nickname)
        .bind(default_role_id)
        .fetch_one(pool)
        .await
    }

    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
    }

    /// Find user by email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    /// Find user by phone
    pub async fn find_by_phone(pool: &PgPool, phone: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE phone = $1")
            .bind(phone)
            .fetch_optional(pool)
            .await
    }

    /// Get user with role information
    pub async fn get_user_with_role(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Option<UserResponse>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT
                u.id, u.email, u.nickname, u.avatar_url,
                u.email_verified, u.created_at, r.name as "role_name?"
            FROM users u
            LEFT JOIN roles r ON u.role_id = r.id
            WHERE u.id = $1
            "#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|row| UserResponse {
            id: row.id,
            email: row.email,
            nickname: row.nickname,
            avatar_url: row.avatar_url,
            role: row.role_name,
            email_verified: row.email_verified.unwrap_or(false),
            created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
        }))
    }

    /// Update user's last login time
    pub async fn update_last_login(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Update user status
    pub async fn update_status(
        pool: &PgPool,
        user_id: Uuid,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Update user role
    pub async fn update_role(
        pool: &PgPool,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET role_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(role_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Update user profile (nickname, avatar_url, phone)
    pub async fn update_profile(
        pool: &PgPool,
        user_id: Uuid,
        nickname: &str,
        avatar_url: Option<&str>,
        phone: Option<&str>,
    ) -> Result<UserResponse, sqlx::Error> {
        // First update the user
        sqlx::query(
            "UPDATE users SET nickname = $1, avatar_url = $2, phone = $3, updated_at = NOW() WHERE id = $4"
        )
        .bind(nickname)
        .bind(avatar_url)
        .bind(phone)
        .bind(user_id)
        .execute(pool)
        .await?;

        // Then fetch and return the updated user
        Self::get_user_with_role(pool, user_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    /// List all users with pagination
    pub async fn list(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserResponse>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                u.id, u.email, u.nickname, u.avatar_url,
                u.email_verified, u.created_at, r.name as "role_name?"
            FROM users u
            LEFT JOIN roles r ON u.role_id = r.id
            ORDER BY u.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| UserResponse {
                id: row.id,
                email: row.email,
                nickname: row.nickname,
                avatar_url: row.avatar_url,
                role: row.role_name,
                email_verified: row.email_verified.unwrap_or(false),
                created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
            })
            .collect())
    }

    /// Delete user
    pub async fn delete(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
