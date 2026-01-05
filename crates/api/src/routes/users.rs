use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use entangle_auth::{create_token, hash_password, verify_password, PermissionService};
use entangle_core::{AppError, AppResult};
use entangle_db::{
    models::{CreateUser, LoginUser, UserResponse},
    RoleRepository, UserRepository,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::AuthUser;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

/// Register a new user
async fn register(
    State(pool): State<PgPool>,
    Json(user_data): Json<CreateUser>,
) -> AppResult<Json<AuthResponse>> {
    // Check if email already exists
    if UserRepository::find_by_email(&pool, &user_data.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    // Check if phone already exists (if provided)
    if let Some(ref phone) = user_data.phone {
        if UserRepository::find_by_phone(&pool, phone)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict("Phone already registered".to_string()));
        }
    }

    // Hash password
    let password_hash = hash_password(&user_data.password)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

    // Get default editor role (allows creating documents)
    let editor_role = RoleRepository::find_by_name(&pool, "editor")
        .await?
        .ok_or_else(|| AppError::Internal("Default editor role not found".to_string()))?;

    // Create user
    let user = UserRepository::create(&pool, &user_data, password_hash, editor_role.id).await?;

    // Get user with role
    let user_response = UserRepository::get_user_with_role(&pool, user.id)
        .await?
        .ok_or_else(|| AppError::Internal("Failed to fetch created user".to_string()))?;

    // Create JWT token
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let token = create_token(user.id, &jwt_secret, 86400 * 7) // 7 days
        .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

    Ok(Json(AuthResponse {
        token,
        user: user_response,
    }))
}

/// Login user
async fn login(
    State(pool): State<PgPool>,
    Json(credentials): Json<LoginUser>,
) -> AppResult<Json<AuthResponse>> {
    // Find user by email
    let user = UserRepository::find_by_email(&pool, &credentials.email)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid email or password".to_string()))?;

    // Check if user is active
    if user.status != "active" {
        return Err(AppError::Forbidden(
            "User account is not active".to_string(),
        ));
    }

    // Verify password
    let password_valid = verify_password(&credentials.password, &user.password_hash)
        .map_err(|e| AppError::Internal(format!("Password verification failed: {}", e)))?;

    if !password_valid {
        return Err(AppError::Auth("Invalid email or password".to_string()));
    }

    // Update last login time
    UserRepository::update_last_login(&pool, user.id).await?;

    // Get user with role
    let user_response = UserRepository::get_user_with_role(&pool, user.id)
        .await?
        .ok_or_else(|| AppError::Internal("Failed to fetch user".to_string()))?;

    // Create JWT token
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let token = create_token(user.id, &jwt_secret, 86400 * 7) // 7 days
        .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

    Ok(Json(AuthResponse {
        token,
        user: user_response,
    }))
}

/// Get current user info
async fn get_me(
    State(pool): State<PgPool>,
    user: AuthUser,
) -> AppResult<Json<UserResponse>> {
    let user_response = UserRepository::get_user_with_role(&pool, user.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user_response))
}

/// Get current user's permissions
async fn get_my_permissions(
    State(pool): State<PgPool>,
    user: AuthUser,
) -> AppResult<Json<Vec<String>>> {
    let permissions = PermissionService::get_user_permissions(&pool, user.user_id).await?;
    Ok(Json(permissions))
}

/// Get user by ID (admin only)
async fn get_user_by_id(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    // Check admin permission
    if !PermissionService::is_admin(&pool, user.user_id).await? {
        return Err(AppError::Forbidden(
            "Admin permission required".to_string(),
        ));
    }

    let user_response = UserRepository::get_user_with_role(&pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user_response))
}

/// List all users (admin only)
async fn list_users(
    State(pool): State<PgPool>,
    user: AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<Vec<UserResponse>>> {
    // Check admin permission
    if !PermissionService::is_admin(&pool, user.user_id).await? {
        return Err(AppError::Forbidden(
            "Admin permission required".to_string(),
        ));
    }

    let users = UserRepository::list(&pool, pagination.limit, pagination.offset).await?;
    Ok(Json(users))
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role_id: Uuid,
}

/// Update user role (admin only)
async fn update_user_role(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> AppResult<StatusCode> {
    // Check admin permission
    if !PermissionService::is_admin(&pool, user.user_id).await? {
        return Err(AppError::Forbidden(
            "Admin permission required".to_string(),
        ));
    }

    // Verify role exists
    RoleRepository::find_by_id(&pool, req.role_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

    UserRepository::update_role(&pool, user_id, req.role_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Update current user's profile
async fn update_my_profile(
    State(pool): State<PgPool>,
    user: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<UserResponse>> {
    let updated_user = UserRepository::update_profile(
        &pool,
        user.user_id,
        &req.nickname,
        req.avatar_url.as_deref(),
        req.phone.as_deref(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to update profile: {}", e)))?;

    Ok(Json(updated_user))
}

/// Public routes (no authentication required)
pub fn public_routes() -> Router<PgPool> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

/// Protected routes (authentication required)
pub fn protected_routes() -> Router<PgPool> {
    Router::new()
        .route("/me", get(get_me))
        .route("/me", put(update_my_profile))
        .route("/me/permissions", get(get_my_permissions))
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user_by_id))
        .route("/users/:id/role", post(update_user_role))
}
