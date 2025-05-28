mod model;
mod routes;
mod requests;

#[cfg(test)]
mod tests;

pub use model::*;
pub use routes::{init_routes, ApiDoc};
