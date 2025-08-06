mod apikey;
mod app_state;
mod authenticated;
mod discord;
mod logout;
mod middleware;
pub mod permission;
mod routes;
mod tests;
mod token;

pub use app_state::init_app_state;
pub use routes::{init_routes, ApiDoc};

pub use authenticated::Authenticated;
pub use middleware::UserAuth;
pub use permission::check_higher_privilege;
pub use permission::Permission;

#[cfg(test)]
pub use app_state::AuthAppState;
#[cfg(test)]
pub use token::create_test_token;
