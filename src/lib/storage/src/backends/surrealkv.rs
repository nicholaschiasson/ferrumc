use crate::errors::StorageError;
use crate::DatabaseBackend;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

pub struct SurrealKVBackend {
    db: Arc<RwLock<surrealkv::Store>>,
}

impl DatabaseBackend for SurrealKVBackend {
    async fn initialize(store_path: Option<PathBuf>) -> Result<Self, StorageError>
    where
        Self: Sized,
    {
        if let Some(path) = store_path {
            let options = surrealkv::Options {
                dir: path,
                disk_persistence: true,
                ..Default::default()
            };
            let db = Arc::new(RwLock::new(
                surrealkv::Store::new(options)
                    .map_err(|e| StorageError::DatabaseInitError(e.to_string()))?,
            ));
            Ok(Self { db })
        } else {
            Err(StorageError::DatabaseInitError(
                "No path provided".to_string(),
            ))
        }
    }

    async fn insert(
        &mut self,
        table: String,
        key: u64,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        if self.exists(table.clone(), key).await? {
            return Err(StorageError::KeyExists(key));
        }
        let mut modified_key = table.as_bytes().to_vec();
        modified_key.extend_from_slice(&key.to_be_bytes());
        let mut tx = self
            .db
            .write()
            .begin()
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        tx.set(&modified_key, &value)
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        Ok(())
    }

    async fn get(&mut self, table: String, key: u64) -> Result<Option<Vec<u8>>, StorageError> {
        let mut modified_key = table.as_bytes().to_vec();
        modified_key.extend_from_slice(&key.to_be_bytes());
        let tx = self
            .db
            .read()
            .begin()
            .map_err(|e| StorageError::ReadError(e.to_string()))?;
        let value = tx
            .get(&modified_key)
            .map_err(|e| StorageError::ReadError(e.to_string()))?;
        Ok(value)
    }

    async fn delete(&mut self, table: String, key: u64) -> Result<(), StorageError> {
        let mut modified_key = table.as_bytes().to_vec();
        modified_key.extend_from_slice(&key.to_be_bytes());
        let mut tx = self
            .db
            .write()
            .begin()
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        tx.delete(&modified_key)
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        Ok(())
    }

    async fn update(
        &mut self,
        table: String,
        key: u64,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        if self.exists(table.clone(), key).await? {
            let mut modified_key = table.as_bytes().to_vec();
            modified_key.extend_from_slice(&key.to_be_bytes());
            let mut tx = self
                .db
                .write()
                .begin()
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            tx.set(&modified_key, &value)
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            tx.commit()
                .await
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            Ok(())
        } else {
            Err(StorageError::KeyNotFound(key))
        }
    }

    async fn upsert(
        &mut self,
        table: String,
        key: u64,
        value: Vec<u8>,
    ) -> Result<bool, StorageError> {
        if self.exists(table.clone(), key).await? {
            self.update(table, key, value).await?;
            Ok(true)
        } else {
            self.insert(table, key, value).await?;
            Ok(false)
        }
    }

    async fn exists(&mut self, table: String, key: u64) -> Result<bool, StorageError> {
        let mut modified_key = table.as_bytes().to_vec();
        modified_key.extend_from_slice(&key.to_be_bytes());
        let tx = self
            .db
            .read()
            .begin()
            .map_err(|e| StorageError::ReadError(e.to_string()))?;
        let value = tx
            .get(&modified_key)
            .map_err(|e| StorageError::ReadError(e.to_string()))?;
        Ok(value.is_some())
    }

    async fn details(&mut self) -> String {
        "SurrealKV 0.3.6".to_string()
    }

    async fn batch_insert(
        &mut self,
        table: String,
        data: Vec<(u64, Vec<u8>)>,
    ) -> Result<(), StorageError> {
        let mut tx = self
            .db
            .write()
            .begin()
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        for (key, value) in data {
            let mut modified_key = table.as_bytes().to_vec();
            modified_key.extend_from_slice(&key.to_be_bytes());
            tx.set(&modified_key, &value)
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
        }
        tx.commit()
            .await
            .map_err(|e| StorageError::WriteError(e.to_string()))?;
        Ok(())
    }

    async fn batch_get(
        &mut self,
        table: String,
        keys: Vec<u64>,
    ) -> Result<Vec<Option<Vec<u8>>>, StorageError> {
        let tx = self
            .db
            .read()
            .begin()
            .map_err(|e| StorageError::ReadError(e.to_string()))?;
        let mut values = Vec::new();
        for key in keys {
            let mut modified_key = table.as_bytes().to_vec();
            modified_key.extend_from_slice(&key.to_be_bytes());
            let value = tx
                .get(&modified_key)
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            values.push(value);
        }
        Ok(values)
    }

    async fn flush(&mut self) -> Result<(), StorageError> {
        Ok(())
    }

    async fn create_table(&mut self, _: String) -> Result<(), StorageError> {
        Ok(())
    }
    async fn close(&mut self) -> Result<(), StorageError> {
        #[allow(clippy::await_holding_lock)]
        let write_guard = self.db.write();
        let res = write_guard.close().await;
        drop(write_guard);
        res.map_err(|e| StorageError::CloseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn setup_backend() -> SurrealKVBackend {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        SurrealKVBackend::initialize(Some(path)).await.unwrap()
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
