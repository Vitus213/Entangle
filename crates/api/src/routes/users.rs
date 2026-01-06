use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use axum_extra::extract::Multipart;
use entangle_auth::{create_token, hash_password, verify_password, PermissionService};
use entangle_core::{AppError, AppResult};
use entangle_db::{
    models::{CreateUser, LoginUser, UserResponse},
    RoleRepository, UserRepository,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

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

#[derive(Debug, Serialize)]
pub struct AvatarUploadResponse {
    pub avatar_url: String,
}

/// Upload avatar image
/// 支持上传图片并保存到本地或返回 base64 URL
async fn upload_avatar(
    State(pool): State<PgPool>,
    user: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<AvatarUploadResponse>> {
    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError::Internal(format!("Failed to read multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("unknown");
        if name == "avatar" {
            let filename = field.file_name()
                .unwrap_or("avatar.jpg")
                .to_string();

            // 获取文件扩展名
            let extension = filename
                .rsplit('.')
                .next()
                .unwrap_or("jpg");

            // 验证文件类型
            if !["jpg", "jpeg", "png", "gif", "webp"].contains(&extension.to_lowercase().as_str()) {
                return Err(AppError::BadRequest(
                    "Invalid file type. Only JPG, PNG, GIF, WebP are supported.".to_string()
                ));
            }

            // 读取文件数据
            let data = field.bytes().await
                .map_err(|e| AppError::Internal(format!("Failed to read file: {}", e)))?;

            // 验证文件大小 (最大 5MB)
            if data.len() > 5 * 1024 * 1024 {
                return Err(AppError::BadRequest(
                    "File too large. Maximum size is 5MB.".to_string()
                ));
            }

            // 简单的图片验证（检查文件头）
            if data.len() < 4 {
                return Err(AppError::BadRequest("Invalid image file".to_string()));
            }

            let is_valid_image = match extension.to_lowercase().as_str() {
                "jpg" | "jpeg" => data.starts_with(&[0xFF, 0xD8, 0xFF]),
                "png" => data.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
                "gif" => data.starts_with(&[0x47, 0x49, 0x46, 0x38]),
                "webp" => data.starts_with(&[0x52, 0x49, 0x46, 0x46]) && data[8..12].starts_with(b"WEBP"),
                _ => false,
            };

            if !is_valid_image {
                return Err(AppError::BadRequest("Invalid image file format".to_string()));
            }

            // 转换为 base64 data URL
            let mime_type = match extension.to_lowercase().as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "webp" => "image/webp",
                _ => "image/jpeg",
            };

            let base64_data = BASE64.encode(&data);
            let avatar_url = format!("data:{};base64,{}", mime_type, base64_data);

            // 获取当前用户信息
            let current_user = UserRepository::find_by_id(&pool, user.user_id)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to fetch user: {}", e)))?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

            // 更新用户头像（保持昵称不变）
            let updated_user = UserRepository::update_profile(
                &pool,
                user.user_id,
                &current_user.nickname, // 保持当前昵称
                Some(&avatar_url),
                None, // 不更新电话
            )
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update avatar: {}", e)))?;

            return Ok(Json(AvatarUploadResponse {
                avatar_url: updated_user.avatar_url.unwrap_or_default(),
            }));
        }
    }

    Err(AppError::BadRequest("No avatar file provided".to_string()))
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
        .route("/me/avatar", post(upload_avatar))
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user_by_id))
        .route("/users/:id/role", post(update_user_role))
}
