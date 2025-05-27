mod routes;
mod model;

#[cfg(test)]
mod tests;

pub use model::*;
pub use routes::{init_routes, ApiDoc};
