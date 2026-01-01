// Real-time collaboration using CRDT (Yrs)

pub mod awareness;
pub mod document;
pub mod sync;

pub use awareness::{
    generate_user_color, AwarenessManager, AwarenessState, CursorPosition, Selection, UserInfo,
};
pub use document::{CollabDocument, CollabError, DocumentMetadata};
pub use sync::{BroadcastMessage, ConnectionInfo, DocumentRoom, RoomManager};
