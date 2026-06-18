mod apikey;
mod authenticated;
pub mod connected_accounts;
pub mod discord;
mod logout;
mod middleware;
pub mod oauth;
pub mod patreon;
pub mod permission;
mod refresh;
mod routes;
mod tests;
mod token;

pub use routes::{init_routes, ApiDoc};

pub use authenticated::Authenticated;
pub use middleware::UserAuth;
pub use oauth::OAuthOptions;
pub use permission::Permission;

#[cfg(test)]
pub use token::create_test_token;
