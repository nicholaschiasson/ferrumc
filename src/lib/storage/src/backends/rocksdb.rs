use crate::errors::StorageError;
use crate::DatabaseBackend;
use parking_lot::RwLock;
use rocksdb::DB;
use std::path::PathBuf;
use std::sync::Arc;

pub struct RocksDBBackend {
    db: Arc<RwLock<DB>>,
}

impl DatabaseBackend for RocksDBBackend {
    async fn initialize(store_path: Option<PathBuf>) -> Result<Self, StorageError>
    where
        Self: Sized,
    {
        let mut options = rocksdb::Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        options.set_compression_options_parallel_threads(4);
        options.set_max_background_jobs(4);
        options.set_max_open_files(1000);
        options.increase_parallelism(4);
        options.set_allow_mmap_writes(true);
        options.set_allow_mmap_reads(true);
        if let Some(path) = store_path {
            let db = DB::open(&options, path)
                .map_err(|e| StorageError::DatabaseInitError(e.to_string()))?;
            Ok(Self {
                db: Arc::new(RwLock::new(db)),
            })
        } else {
            Err(StorageError::InvalidPath)
        }
    }

    async fn insert(
        &mut self,
        table: String,
        key: u64,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let db = db.read();
            let cf = db.cf_handle(&table).unwrap();
            db.put_cf(cf, key.to_be_bytes(), &value)
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            Ok::<(), StorageError>(())
        })
            .await
            .expect("Failed to insert data into database")?;
        Ok(())
    }

    async fn get(&mut self, table: String, key: u64) -> Result<Option<Vec<u8>>, StorageError> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let db = db.read();
            let cf = db.cf_handle(&table).unwrap();
            let value = db
                .get_cf(cf, key.to_be_bytes())
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            if let Some(value) = value {
                Ok(Some(value.to_vec()))
            } else {
                Ok(None)
            }
        })
            .await
            .expect("Failed to get data from database")?;
        Ok(result)
    }

    async fn delete(&mut self, table: String, key: u64) -> Result<(), StorageError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let db = db.read();
            let cf = db.cf_handle(&table).unwrap();
            db.delete_cf(cf, key.to_be_bytes())
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            Ok::<(), StorageError>(())
        })
            .await
            .expect("Failed to delete data from database")?;
        Ok(())
    }

    async fn update(
        &mut self,
        table: String,
        key: u64,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let db = db.read();
            let cf = db.cf_handle(&table).unwrap();
            db.put_cf(cf, key.to_be_bytes(), &value)
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            Ok::<(), StorageError>(())
        })
            .await
            .expect("Failed to update data in database")?;
        Ok(())
    }

    async fn upsert(
        &mut self,
        table: String,
        key: u64,
        value: Vec<u8>,
    ) -> Result<bool, StorageError> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let db = db.read();
            let cf = db.cf_handle(&table).unwrap();
            db.put_cf(cf, key.to_be_bytes(), &value)
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            if let Ok(Some(_)) = db.get_cf(cf, key.to_be_bytes()) {
                Ok(true)
            } else {
                Ok(false)
            }
        })
            .await
            .expect("Failed to upsert data in database")?;
        Ok(result)
    }

    async fn exists(&mut self, table: String, key: u64) -> Result<bool, StorageError> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let db = db.read();
            let cf = db.cf_handle(&table).unwrap();
            let value = db
                .get_cf(cf, key.to_be_bytes())
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            Ok(value.is_some())
        })
            .await
            .expect("Failed to check if key exists in database")?;
        Ok(result)
    }

    async fn details(&mut self) -> String {
        "RocksDB 0.22.0".to_string()
    }

    async fn batch_insert(
        &mut self,
        table: String,
        data: Vec<(u64, Vec<u8>)>,
    ) -> Result<(), StorageError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let db = db.read();
            let cf = db.cf_handle(&table).unwrap();
            let mut batch = rocksdb::WriteBatch::default();
            for (key, value) in data {
                batch.put_cf(cf, key.to_be_bytes(), &value);
            }
            db.write(batch)
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            Ok::<(), StorageError>(())
        })
            .await
            .expect("Failed to batch insert data into database")?;
        Ok(())
    }

    async fn batch_get(
        &mut self,
        table: String,
        keys: Vec<u64>,
    ) -> Result<Vec<Option<Vec<u8>>>, StorageError> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let db = db.read();
            let cf = db.cf_handle(&table).unwrap();
            let mut values = Vec::new();
            for key in keys {
                let value = db
                    .get_cf(cf, key.to_be_bytes())
                    .map_err(|e| StorageError::ReadError(e.to_string()))?;
                if let Some(value) = value {
                    values.push(Some(value.to_vec()));
                } else {
                    values.push(None);
                }
            }
            Ok(values)
        })
            .await
            .expect("Failed to batch get data from database")?;
        Ok(result)
    }

    async fn flush(&mut self) -> Result<(), StorageError> {
        self.db
            .read()
            .flush()
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        Ok(())
    }

    async fn create_table(&mut self, table: String) -> Result<(), StorageError> {
        self.db
            .write()
            .create_cf(&table, &rocksdb::Options::default())
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        Ok(())
    }

    async fn close(&mut self) -> Result<(), StorageError> {
        self.flush().await?;
        self.db
            .read()
            .flush_wal(true)
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn setup_backend() -> RocksDBBackend {
        let db_file = tempdir().unwrap().into_path();
        let path = db_file.join("test");
        let mut backend = RocksDBBackend::initialize(Some(path)).await.unwrap();
        backend.create_table("test_table".to_string()).await.unwrap();
        backend
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let mut backend = setup_backend().await;
        let table = "test_table".to_string();
        let key = 0;
        let value = b"test_value_for_insert_and_get".to_vec();

        backend.insert(table.clone(), key, value.clone()).await.unwrap();
        let retrieved_value = backend.get(table, key).await.unwrap().unwrap();
        assert_eq!(retrieved_value, value);
    }

    #[tokio::test]
    async fn test_delete() {
        let mut backend = setup_backend().await;
        let table = "test_table".to_string();
        let key = 1;
        let value = b"test_value_for_delete".to_vec();

        backend.insert(table.clone(), key, value.clone()).await.unwrap();
        backend.delete(table.clone(), key).await.unwrap();
        let retrieved_value = backend.get(table, key).await.unwrap();
        assert!(retrieved_value.is_none());
    }

    #[tokio::test]
    async fn test_update() {
        let mut backend = setup_backend().await;
        let table = "test_table".to_string();
        let key = 2;
        let value = b"test_value_for_update".to_vec();
        let new_value = b"new_value_for_update".to_vec();

        backend.insert(table.clone(), key, value.clone()).await.unwrap();
        backend.update(table.clone(), key, new_value.clone()).await.unwrap();
        let retrieved_value = backend.get(table, key).await.unwrap().unwrap();
        assert_eq!(retrieved_value, new_value);
    }

    #[tokio::test]
    async fn test_upsert() {
        let mut backend = setup_backend().await;
        let table = "test_table".to_string();
        let key = 3;
        let value = b"test_value_for_upsert".to_vec();
        let new_value = b"new_value_for_upsert".to_vec();

        let inserted = backend.upsert(table.clone(), key, value.clone()).await.unwrap();
        assert!(!inserted);
        let updated = backend.upsert(table.clone(), key, new_value.clone()).await.unwrap();
        assert!(updated);
        let retrieved_value = backend.get(table, key).await.unwrap().unwrap();
        assert_eq!(retrieved_value, new_value);
    }
}
