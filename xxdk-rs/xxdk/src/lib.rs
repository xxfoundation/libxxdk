mod util;

pub mod base;
pub mod rpc;

#[doc(inline)]
pub use base::{get_dependencies, get_git_version, get_version};
