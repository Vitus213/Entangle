pub mod users;
pub mod documents;

pub use users::{protected_routes as user_protected_routes, public_routes as user_public_routes};
pub use documents::document_routes;
