use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use rocksdb::{ColumnFamilyDescriptor, DB};
use tokio::fs;
use tracing::{debug, info};
use crate::utils::config::get_global_config;
use crate::utils::error::Error;

pub mod chunks;

pub struct Database {
    pub db: Arc<DB>,
}

pub async fn start_database() -> Result<Database, Error> {
    let root = if env::var("FERRUMC_ROOT").is_ok() {
        PathBuf::from(env::var("FERRUMC_ROOT").unwrap())
    } else {
        PathBuf::from(
            env::current_exe()
                .unwrap()
                .parent()
                .ok_or(Error::Generic("Failed to get exe directory".to_string()))?,
        )
    };

    let world = get_global_config().world.clone();
    let world_path = root.join("data").join(world);

    debug!("Opening database at {:?}", world_path);

    if !fs::try_exists(&world_path).await? {
        fs::create_dir_all(&world_path).await?;
    }

    let mut opts = rocksdb::Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    // Optimize for multithreaded access
    opts.increase_parallelism(num_cpus::get() as i32);
    opts.set_max_background_jobs(num_cpus::get() as i32);


    // Define all column families you'll use
    let cf_names = vec!["chunks/overworld", "chunks/nether", "chunks/end"];
    let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_names
        .iter()
        .map(|name| {
            let mut cf_opts = rocksdb::Options::default();
            cf_opts.set_max_write_buffer_number(128);
            ColumnFamilyDescriptor::new(name.to_string(), cf_opts)
        })
        .collect();

    let database = DB::open_cf_descriptors(&opts, world_path, cf_descriptors)
        .map_err(|e| Error::DatabaseError(format!("Failed to open database: {}", e)))?;

    info!("Database started");
    /*let mut database = DB::open(&opts, world_path)
        .map_err(|e| Error::DatabaseError(format!("Failed to open database: {}", e)))?;

    info!("Database started");

    let cf_name = "chunks/overworld".to_string();
    if !database.cf_handle(&cf_name).is_some() {
        let mut cf_opts = rocksdb::Options::default();
        cf_opts.set_max_write_buffer_number(16);
        database.create_cf(cf_name.clone(), &cf_opts)
            .expect("Failed to create CF");
    }
*/

    Ok(Database { db: Arc::new(database) })
}
