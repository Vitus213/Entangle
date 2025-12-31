// Database access layer
pub mod models;
pub mod pool;
pub mod repository;

pub use pool::create_pool;
pub use repository::{DocumentRepository, PermissionRepository, RoleRepository, UserRepository};
