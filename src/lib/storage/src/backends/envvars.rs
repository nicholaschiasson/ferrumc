use std::path::PathBuf;
use crate::DatabaseBackend;
use crate::errors::StorageError;

pub struct EnvVarsBackend;

/// Please for the love of god don't actually use this. It's entirely for shits and giggles.
impl DatabaseBackend for EnvVarsBackend {
    async fn initialize(_: Option<PathBuf>) -> Result<Self, StorageError>
    where
        Self: Sized
    {
        Ok(Self)
    }

    async fn insert(&mut self, table: String, key: u64, value: Vec<u8>) -> Result<(), StorageError> {
        let key_str = key.to_string() + &table;
        let val = base64::encode(&value);
        std::env::set_var(key_str, val);
        Ok(())
    }

    async fn get(&mut self, table: String, key: u64) -> Result<Option<Vec<u8>>, StorageError> {
        let key_str = key.to_string() + &table;
        match std::env::var(key_str) {
            Ok(val) => {
                let decoded = base64::decode(&val).map_err(|e| StorageError::ReadError(e.to_string()))?;
                Ok(Some(decoded))
            }
            Err(_) => Ok(None)
        }
    }

    async fn delete(&mut self, table: String, key: u64) -> Result<(), StorageError> {
        let key_str = key.to_string() + &table;
        std::env::remove_var(key_str);
        Ok(())
    }

    async fn update(&mut self, table: String, key: u64, value: Vec<u8>) -> Result<(), StorageError> {
        if self.exists(table.clone(), key).await? {
            self.insert(table, key, value).await
        } else {
            Err(StorageError::KeyNotFound(key))
        }
    }

    async fn upsert(&mut self, table: String, key: u64, value: Vec<u8>) -> Result<bool, StorageError> {
        let did_exist = self.exists(table.clone(), key).await?;
        self.insert(table, key, value).await?;
        Ok(did_exist)
    }

    async fn exists(&mut self, table: String, key: u64) -> Result<bool, StorageError> {
        let key_str = key.to_string() + &table;
        match std::env::var(key_str) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false)
        }
    }

    async fn details(&mut self) -> String {
        "EnvVars".to_string()
    }

    async fn batch_insert(&mut self, table: String, data: Vec<(u64, Vec<u8>)>) -> Result<(), StorageError> {
        for (key, value) in data {
            self.insert(table.clone(), key, value).await?;
        }
        Ok(())
    }

    async fn batch_get(&mut self, table: String, keys: Vec<u64>) -> Result<Vec<Option<Vec<u8>>>, StorageError> {
        let mut values = Vec::new();
        for key in keys {
            values.push(self.get(table.clone(), key).await?);
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn setup_backend() -> EnvVarsBackend {
        let db_file = tempdir().unwrap().into_path();
        let path = db_file.join("test.db");
        let mut backend = EnvVarsBackend::initialize(Some(path)).await.unwrap();
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