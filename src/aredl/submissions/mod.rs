mod model;
mod get;
mod post;
mod delete;
mod status;
mod patch;
mod routes;

pub use model::*;
pub use get::*;
pub use patch::*;
pub use post::*;
pub use routes::{init_routes, ApiDoc};
