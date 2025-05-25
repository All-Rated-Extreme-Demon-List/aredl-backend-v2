mod model;
mod routes;

#[cfg(test)]
pub mod tests;

pub use model::*;
pub use routes::{init_routes, ApiDoc};
