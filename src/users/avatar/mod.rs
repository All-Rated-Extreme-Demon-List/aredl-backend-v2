mod model;
mod routes;

#[cfg(test)]
mod tests;

pub use model::{AvatarRefresher, RefreshError};
pub use routes::{init_routes, ApiDoc};
