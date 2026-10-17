pub(crate) mod patreon;
mod probe;
mod redirect;
mod routes;

pub(crate) use redirect::{redirect_prefix, redirect_route};
pub use routes::{init_routes, ApiDoc};
