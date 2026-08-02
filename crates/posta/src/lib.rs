mod error;
mod model;
mod store;
mod types;

pub use error::Error;
pub use store::Store;
pub use types::{Accept, Accepted, Kind, Letter, Post, Replied, Reply, Staged};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn graph() -> keel::Graph {
    model::graph()
}
