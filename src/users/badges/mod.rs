mod badges_list;
mod model;
mod routes;
mod statistics;
#[cfg(test)]
mod tests;

pub use model::*;
pub use routes::{init_routes, ApiDoc};
