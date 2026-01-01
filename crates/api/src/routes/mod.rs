pub mod users;
pub mod documents;
pub mod folders;
pub mod tags;

pub use users::{protected_routes as user_protected_routes, public_routes as user_public_routes};
pub use documents::document_routes;
pub use folders::folder_routes;
pub use tags::tag_routes;
