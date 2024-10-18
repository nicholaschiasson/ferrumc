#[cfg(feature = "redb")]
pub mod redb;
#[cfg(feature = "rocksdb")]
pub mod rocksdb;

#[cfg(feature = "sled")]
pub mod sled;

#[cfg(feature = "surrealkv")]
pub mod surrealkv;

#[cfg(feature = "envvar")]
mod envvars;

#[cfg(not(any(feature = "redb", feature = "rocksdb", feature = "sled", feature = "surrealkv", feature = "envvar")))]
compile_error!("At least one storage backend must be enabled");
