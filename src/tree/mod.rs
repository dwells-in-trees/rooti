pub mod node;
pub mod growth;
pub mod biology;

// Re-exports
pub use node::{ Tree, TreeConfig };

// Null sentinel for parent root node
pub(crate) const NULL_IDX: u32 = u32::MAX;