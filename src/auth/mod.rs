mod apikey;
mod authenticated;
pub mod discord;
mod logout;
mod middleware;
pub mod permission;
mod routes;
mod tests;
mod token;

pub use routes::{init_routes, ApiDoc};

pub use authenticated::Authenticated;
pub use middleware::UserAuth;
pub use permission::check_higher_privilege_user;
pub use permission::Permission;

#[cfg(test)]
pub use token::create_test_token;
