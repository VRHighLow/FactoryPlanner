//! Era 1 external data packs — interned string IDs, boot validation, craft/tech lookups.

mod registry;
mod tech;
mod types;

pub use registry::{content, init_content, try_content};
pub use tech::TechState;
pub use types::*;
