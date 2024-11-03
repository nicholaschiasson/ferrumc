pub mod errors;
mod importing;
mod vanilla_chunk_format;
mod db_functions;
mod chunk_format;

use crate::errors::WorldError;
use ferrumc_storage::compressors::Compressor;
use ferrumc_storage::DatabaseBackend;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;
use bitcode_derive::{Decode, Encode};
use tokio::fs::create_dir_all;
use tracing::{error, info, warn};
use type_hash::TypeHash;
use ferrumc_config::statics::get_global_config;
use crate::chunk_format::Chunk;

#[derive(Clone)]
pub struct World {
    storage_backend: Arc<Box<dyn DatabaseBackend + Send + Sync>>,
    compressor: Compressor,
    // TODO: Cache
}

async fn check_config_validity() -> Result<(), WorldError> {
    // We don't actually check if the import path is valid here since that would brick a server
    // if the world is imported then deleted after the server starts. Those checks are handled in
    // the importing logic.

    let config = get_global_config();
    if config.database.backend.is_empty() {
        error!("No backend specified. Please set the backend in the configuration file.");
        return Err(WorldError::InvalidBackend(config.database.backend.clone()));
    }
    if !Path::new(&config.database.db_path).exists() {
        warn!("World path does not exist. Attempting to create it.");
        if create_dir_all(&config.database.db_path).await.is_err() {
            error!("Could not create world path: {}", config.database.db_path);
            return Err(WorldError::InvalidWorldPath(
                config.database.db_path.clone(),
            ));
        }
    }
    if Path::new(&config.database.db_path).is_file() {
        error!("World path is a file. Please set the world path to a directory.");
        return Err(WorldError::InvalidWorldPath(
            config.database.db_path.clone(),
        ));
    }
    if let Err(e) = Path::new(&config.database.db_path).read_dir() {
        error!("Could not read world path: {}", e);
        return Err(WorldError::InvalidWorldPath(
            config.database.db_path.clone(),
        ));
    }

    if config.database.compression.is_empty() {
        error!("No compressor specified. Please set the compressor in the configuration file.");
        return Err(WorldError::InvalidCompressor(
            config.database.compression.clone(),
        ));
    }
    if config.database.import_path.is_empty() {
        error!("No import path specified. Please set the import path in the configuration file.");
        return Err(WorldError::InvalidImportPath(
            config.database.import_path.clone(),
        ));
    }
    Ok(())
}
#[derive(Encode, Decode)]
pub(crate) struct DBMetadata {
    pub(crate) compressor: String,
    pub(crate) backend: String,
    pub(crate) world_format: u64,
}

impl World {
    /// Creates a new world instance.
    /// 
    /// You'd probably want to call this at the start of your program. And then use the returned
    /// in a state struct or something.
    pub async fn new() -> Self {
        if let Err(e) = check_config_validity().await {
            error!("Fatal error in database config: {}", e);
            exit(1);
        }
        // Clones are kinda ok here since this is only run once at startup.
        let backend_string = get_global_config().database.backend.trim();
        let backend_path = get_global_config().database.db_path.clone();
        let storage_backend: Result<Box<dyn DatabaseBackend + Send + Sync>, WorldError> = match backend_string
            .to_lowercase()
            .as_str()
        {
            "surrealkv" => {
                #[cfg(feature = "surrealkv")]
                match ferrumc_storage::backends::surrealkv::SurrealKVBackend::initialize(Some(
                    PathBuf::from(&backend_path),
                ))
                .await
                {
                    Ok(backend) => Ok(Box::new(backend)),
                    Err(e) => Err(WorldError::InvalidBackend(e.to_string())),
                }
                #[cfg(not(feature = "surrealkv"))]
                {
                    error!("SurrealKV backend is not enabled. Please enable the 'surrealkv' feature in the Cargo.toml file.");
                    exit(1);
                }
            }
            "sled" => {
                #[cfg(feature = "sled")]
                match ferrumc_storage::backends::sled::SledBackend::initialize(Some(PathBuf::from(
                    &backend_path,
                )))
                .await
                {
                    Ok(backend) => Ok(Box::new(backend)),
                    Err(e) => Err(WorldError::InvalidBackend(e.to_string())),
                }
                #[cfg(not(feature = "sled"))]
                {
                    error!("Sled backend is not enabled. Please enable the 'sled' feature in the Cargo.toml file.");
                    exit(1);
                }
            }
            "rocksdb" => {
                #[cfg(feature = "rocksdb")]
                match ferrumc_storage::backends::rocksdb::RocksDBBackend::initialize(Some(
                    PathBuf::from(backend_path),
                ))
                .await
                {
                    Ok(backend) => Ok(Box::new(backend)),
                    Err(e) => Err(WorldError::InvalidBackend(e.to_string())),
                }
                #[cfg(not(feature = "rocksdb"))]
                {
                    error!("RocksDB backend is not enabled. Please enable the 'rocksdb' feature in the Cargo.toml file.");
                    exit(1);
                }
            }
            "redb" => {
                #[cfg(feature = "redb")]
                match ferrumc_storage::backends::redb::RedbBackend::initialize(Some(PathBuf::from(
                    &backend_path,
                )))
                .await
                {
                    Ok(backend) => Ok(Box::new(backend)),
                    Err(e) => Err(WorldError::InvalidBackend(e.to_string())),
                }
                #[cfg(not(feature = "redb"))]
                {
                    error!("Redb backend is not enabled. Please enable the 'redb' feature in the Cargo.toml file.");
                    exit(1);
                }
            }
            _ => {
                error!(
                    "Invalid storage backend: {}",
                    get_global_config().database.backend
                );
                exit(1);
            }
        };
        let storage_backend = if let Ok(backend) = storage_backend {
            backend
        } else {
            exit(1);
        };

        let compressor_string = get_global_config().database.compression.trim();
        
        info!("Using {} compression algorithm", compressor_string);

        let compression_algo = match compressor_string.to_lowercase().as_str() {
            "zstd" => Compressor::create(
                ferrumc_storage::compressors::CompressorType::Zstd,
                get_global_config().database.compression_level as u32,
            ),
            "brotli" => Compressor::create(
                ferrumc_storage::compressors::CompressorType::Brotli,
                get_global_config().database.compression_level as u32,
            ),
            "deflate" => Compressor::create(
                ferrumc_storage::compressors::CompressorType::Deflate,
                get_global_config().database.compression_level as u32,
            ),
            "gzip" => Compressor::create(
                ferrumc_storage::compressors::CompressorType::Gzip,
                get_global_config().database.compression_level as u32,
            ),
            "zlib" => Compressor::create(
                ferrumc_storage::compressors::CompressorType::Zlib,
                get_global_config().database.compression_level as u32,
            ),
            _ => {
                error!(
                    "Invalid compression algorithm: {}",
                    get_global_config().database.compression
                );
                exit(1);
            }
        };
        
        let metadata = DBMetadata {
            compressor: compressor_string.to_string(),
            backend: backend_string.to_string(),
            world_format: Chunk::type_hash(),
        };
        
        if PathBuf::from(&backend_path).join("META").exists() {
            tokio::fs::read(PathBuf::from(&backend_path).join("META"))
                .await
                .map(|data| {
                    let decoded: DBMetadata = bitcode::decode(&data).unwrap();
                    if decoded.compressor != metadata.compressor
                        || decoded.backend != metadata.backend
                        || decoded.world_format != metadata.world_format
                    {
                        error!("Database metadata does not match configuration. Please delete the database and try again.");
                        exit(1);
                    }
                })
                .unwrap_or_else(|e| {
                    error!("Could not read database metadata: {}", e);
                    exit(1);
                });
        } else {
            match tokio::fs::write(PathBuf::from(&backend_path).join("META"), bitcode::encode(&metadata)).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Could not store database metadate: {}", e);
                    exit(1);
                }
            }
        }

        World {
            storage_backend: Arc::new(storage_backend),
            compressor: compression_algo,
        }
    }
}
