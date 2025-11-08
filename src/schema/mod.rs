mod aredl_schema;
mod arepl_schema;
mod public_schema;

pub use public_schema::generated::public::*;

pub mod aredl {
    pub use super::aredl_schema::custom::*;
    pub use super::aredl_schema::generated::aredl::*;
}

pub mod arepl {
    pub use super::arepl_schema::custom::*;
    pub use super::arepl_schema::generated::arepl::*;
}
