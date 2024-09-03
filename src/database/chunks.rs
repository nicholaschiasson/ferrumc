use rocksdb::{ColumnFamilyDescriptor, Options};
use tracing::{debug, trace, warn};

use crate::database::Database;
use crate::utils::error::Error;
use crate::utils::hash::hash;
use crate::world::chunkformat::Chunk;

impl Database {
    pub async fn insert_chunk(&self, value: Chunk, dimension: String) -> Result<bool, Error> {
        let mut db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let key = hash((value.x_pos, value.z_pos));
            let encoded = bincode::encode_to_vec(value, bincode::config::standard())
                .expect("Failed to encode");

           /* if !db.cf_handle(&cf_name).is_some() {
                let mut cf_opts = Options::default();
                cf_opts.set_max_write_buffer_number(16);
                db.create_cf(cf_name.clone(), &cf_opts)
                    .expect("Failed to create CF");
            }*/
            let cf_name = format!("chunks/{}", dimension);
            if db.cf_handle(&cf_name).is_none() {
                /*let mut cf_opts = Options::default();
                cf_opts.set_max_write_buffer_number(16);
                db.create_cf(cf_name.clone(), &cf_opts)
                    .expect("Failed to create CF");*/
                panic!("CF for dimension {} not found", dimension);
            }

            // Get the column family handle
            let cf_handle = db.cf_handle(&cf_name).expect("CF not found");

            // Insert the key-value pair
            db.put_cf(cf_handle, key, encoded)
                .expect("Failed to insert chunk");
        })
        .await;

        match result {
            Ok(_) => Ok(false),
            Err(e) => {
                warn!("Failed to insert chunk: {}", e);
                Err(Error::DatabaseError("Failed to insert chunk".to_string()))
            }
        }
    }

    pub async fn get_chunk(
        &self,
        x: i32,
        z: i32,
        dimension: impl Into<String>,
    ) -> Result<Option<Chunk>, rocksdb::Error> {
        let dimension = dimension.into();
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let key = hash((x, z));
            trace!("Getting chunk: {}, {}", x, z);

            let cf_name = format!("chunks/{}", dimension);
            let cf_handle = match db.cf_handle(&cf_name) {
                Some(handle) => handle,
                None => return Ok(None), // CF doesn't exist, so chunk doesn't exist
            };

            match db.get_cf(cf_handle, key)? {
                Some(chunk_data) => {
                    let (chunk, len) = bincode::decode_from_slice(&chunk_data, bincode::config::standard())
                        .expect("Failed to decode chunk data");
                    trace!("Got chunk: {} {}, {} bytes long", x, z, len);
                    Ok(Some(chunk))
                }
                None => {
                    debug!("Could not find chunk {}, {}", x, z);
                    Ok(None)
                }
            }
        })
            .await
            .expect("Failed to join tasks")?;
        Ok(result)
    }


    pub async fn chunk_exists(&self, x: i32, z: i32, dimension: String) -> Result<bool, rocksdb::Error> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let key = hash((x, z));
            let cf_name = format!("chunks/{}", dimension);

            if let Some(cf_handle) = db.cf_handle(&cf_name) {
                db.get_cf(cf_handle, key).map(|opt| opt.is_some())
            } else {
                Ok(false) // If the CF doesn't exist, the chunk doesn't exist
            }
        })
            .await
            .expect("Failed to join tasks")?;
        Ok(result)
    }
    pub async fn update_chunk(&self, value: Chunk, dimension: String) -> Result<bool, rocksdb::Error> {
        let mut db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let key = hash((value.x_pos, value.z_pos));
            let encoded = bincode::encode_to_vec(value, bincode::config::standard())
                .expect("Failed to encode");

            let cf_name = format!("chunks/{}", dimension);
            if db.cf_handle(&cf_name).is_none() {
                /*let mut cf_opts = Options::default();
                cf_opts.set_max_write_buffer_number(16);
                db.create_cf(cf_name.clone(), &cf_opts)
                    .expect("Failed to create CF");*/
                panic!("CF for dimension {} not found", dimension);
            }

            let cf_handle = db.cf_handle(&cf_name).expect("CF not found");
            db.put_cf(cf_handle, key, encoded)?;
            Ok(true)
        })
            .await
            .expect("Failed to join tasks")?;
        Ok(result)
    }
}

#[tokio::test]
#[ignore]
async fn dump_chunk() {
    use crate::utils::setup_logger;
    use tokio::net::TcpListener;
    setup_logger().unwrap();
    let state = crate::create_state(TcpListener::bind("0.0.0.0:0").await.unwrap())
        .await
        .unwrap();
    let chunk = state
        .database
        .get_chunk(0, 0, "overworld".to_string())
        .await
        .unwrap()
        .unwrap();
    let outfile = std::fs::File::create("chunk.json").unwrap();
    let mut writer = std::io::BufWriter::new(outfile);
    serde_json::to_writer(&mut writer, &chunk).unwrap();
}
