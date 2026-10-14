mod model;
mod routes;

#[cfg(test)]
pub(super) mod test_utils;
#[cfg(test)]
mod tests;

pub use model::*;
pub use routes::{init_routes, ApiDoc};
