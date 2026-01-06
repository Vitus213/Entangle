use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use entangle_auth::{verify_token, Claims};
use entangle_db::UserRepository;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

/// Authenticated user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub claims: Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract JWT secret from environment or config
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());

        // Try to get token from Authorization header first
        let token = if let Ok(TypedHeader(Authorization(bearer))) =
            parts.extract::<TypedHeader<Authorization<Bearer>>>().await
        {
            bearer.token().to_string()
        } else {
            // Fallback to query parameter (for WebSocket connections)
            let query = parts.extract::<Query<TokenQuery>>().await.map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing Authorization header or token query parameter".to_string(),
                )
            })?;

            query.token.clone().ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing token in query parameters".to_string(),
                )
            })?
        };

        // Verify token
        let claims = verify_token(&token, &jwt_secret).map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                format!("Invalid token: {}", e),
            )
        })?;

        // Parse user_id from claims
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid user ID in token".to_string(),
            )
        })?;

        // Extract database pool from state
        let pool = PgPool::from_ref(state);

        // Verify user exists and is active
        let user = UserRepository::find_by_id(&pool, user_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?
            .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

        if user.status != "active" {
            return Err((
                StatusCode::FORBIDDEN,
                "User account is not active".to_string(),
            ));
        }

        Ok(AuthUser { user_id, claims })
    }
}

/// Macro to create permission guard middleware
#[macro_export]
macro_rules! require_permission {
    ($permission:expr) => {{
        |State(pool): State<PgPool>, user: AuthUser| async move {
            use entangle_auth::PermissionService;
            use entangle_core::AppError;

            let has_permission = PermissionService::has_permission(&pool, user.user_id, $permission)
                .await
                .map_err(|e| AppError::Database(e))?;

            if !has_permission {
                return Err(AppError::Forbidden(format!(
                    "Permission '{}' required",
                    $permission
                )));
            }

            Ok::<_, AppError>(())
        }
    }};
}
