use crate::errors::StorageError;
use crate::DatabaseBackend;
use parking_lot::RwLock;
use redb::TableDefinition;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct RedbBackend {
    db: Arc<RwLock<redb::Database>>,
}

impl DatabaseBackend for RedbBackend {
    async fn initialize(store_path: Option<PathBuf>) -> Result<Self, StorageError> {
        if let Some(path) = store_path {
            let db = if path.exists() {
                redb::Database::open(path)
                    .map_err(|e| StorageError::DatabaseInitError(e.to_string()))?
            } else {
                redb::Database::create(path)
                    .map_err(|e| StorageError::DatabaseInitError(e.to_string()))?
            };
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
        if self.exists(table.clone(), key).await? {
            return Err(StorageError::KeyExists(key));
        }
        tokio::task::spawn_blocking(move || {
            let table_def: TableDefinition<u64, &[u8]> = TableDefinition::new(&table);
            {
                let tx = db
                    .read()
                    .begin_write()
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;
                {
                    let mut open_table = tx
                        .open_table(table_def)
                        .map_err(|e| StorageError::WriteError(e.to_string()))?;

                    open_table
                        .insert(key, value.as_slice())
                        .map_err(|e| StorageError::WriteError(e.to_string()))?;
                }
                tx.commit()
                    .map_err(|e| StorageError::CommitError(e.to_string()))?;
                Ok::<(), StorageError>(())
            }
        })
        .await
        .expect("Failed to insert data into database")?;
        Ok(())
    }

    async fn get(&mut self, table: String, key: u64) -> Result<Option<Vec<u8>>, StorageError> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let table_def: TableDefinition<u64, &[u8]> = TableDefinition::new(&table);
            let tx = db
                .read()
                .begin_read()
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            let open_table = tx
                .open_table(table_def)
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            let value = open_table
                .get(key)
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            if let Some(value) = value {
                Ok(Some(value.value().to_vec()))
            } else {
                Ok(None)
            }
        })
        .await
        .expect("Failed to spawn task")?;
        Ok(result)
    }

    async fn delete(&mut self, table: String, key: u64) -> Result<(), StorageError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let table_def: TableDefinition<u64, &[u8]> = TableDefinition::new(&table);
            let tx = db
                .read()
                .begin_write()
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            #[allow(unused_assignments)]
            let mut did_exist = false;
            {
                let mut open_table = tx
                    .open_table(table_def)
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;
                let value = open_table
                    .remove(key)
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;
                did_exist = value.is_some();
            }
            tx.commit()
                .map_err(|e| StorageError::CommitError(e.to_string()))?;
            if did_exist {
                Ok(())
            } else {
                Err(StorageError::KeyNotFound(key))
            }
        })
        .await
        .expect("Failed to spawn task")
    }

    async fn update(
        &mut self,
        table: String,
        key: u64,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let table_def: TableDefinition<u64, &[u8]> = TableDefinition::new(&table);
            let tx = db
                .read()
                .begin_write()
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            {
                let mut open_table = tx
                    .open_table(table_def)
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;

                let res = open_table
                    .insert(key, value.as_slice())
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;
                if res.is_none() {
                    return Err(StorageError::KeyNotFound(key));
                }
            }
            tx.commit()
                .map_err(|e| StorageError::CommitError(e.to_string()))?;
            Ok(())
        })
        .await
        .expect("Failed to spawn task")
        .map_err(|e| StorageError::UpdateError(e.to_string()))
    }

    async fn upsert(
        &mut self,
        table: String,
        key: u64,
        value: Vec<u8>,
    ) -> Result<bool, StorageError> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let table_def: TableDefinition<u64, &[u8]> = TableDefinition::new(&table);
            let mut did_exist = false;
            let tx = db
                .read()
                .begin_write()
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            {
                let mut open_table = tx
                    .open_table(table_def)
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;

                let res = open_table
                    .insert(key, value.as_slice())
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;
                if res.is_some() {
                    did_exist = true;
                }
            }
            tx.commit()
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            Ok(did_exist)
        })
        .await
        .expect("Failed to spawn task")?;
        Ok(result)
    }

    async fn exists(&mut self, table: String, key: u64) -> Result<bool, StorageError> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let table_def: TableDefinition<u64, &[u8]> = TableDefinition::new(&table);
            let tx = db
                .read()
                .begin_read()
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            let open_table = tx
                .open_table(table_def)
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            let value = open_table
                .get(key)
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            Ok(value.is_some())
        })
        .await
        .expect("Failed to spawn task")?;
        Ok(result)
    }

    async fn details(&mut self) -> String {
        "Redb 2.1.3".to_string()
    }

    async fn batch_insert(
        &mut self,
        table: String,
        data: Vec<(u64, Vec<u8>)>,
    ) -> Result<(), StorageError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let table_def: TableDefinition<u64, &[u8]> = TableDefinition::new(&table);
            {
                let tx = db
                    .read()
                    .begin_write()
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;
                {
                    let mut open_table = tx
                        .open_table(table_def)
                        .map_err(|e| StorageError::WriteError(e.to_string()))?;
                    for (key, value) in data {
                        open_table
                            .insert(key, value.as_slice())
                            .map_err(|e| StorageError::WriteError(e.to_string()))?;
                    }
                }
                tx.commit()
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;
                Ok::<(), StorageError>(())
            }
        })
        .await
        .expect("Failed to insert data into database")?;
        Ok(())
    }

    async fn batch_get(
        &mut self,
        table: String,
        keys: Vec<u64>,
    ) -> Result<Vec<Option<Vec<u8>>>, StorageError> {
        let db = self.db.clone();
        let result = tokio::task::spawn_blocking(move || {
            let table_def: TableDefinition<u64, &[u8]> = TableDefinition::new(&table);
            let tx = db
                .read()
                .begin_read()
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            let open_table = tx
                .open_table(table_def)
                .map_err(|e| StorageError::ReadError(e.to_string()))?;
            let mut values = Vec::new();
            for key in keys {
                let value = open_table
                    .get(key)
                    .map_err(|e| StorageError::ReadError(e.to_string()))?;
                if let Some(value) = value {
                    values.push(Some(value.value().to_vec()));
                } else {
                    values.push(None);
                }
            }
            Ok(values)
        })
        .await
        .expect("Failed to spawn task")?;
        Ok(result)
    }

    async fn flush(&mut self) -> Result<(), StorageError> {
        let db = self.db.clone();
        match tokio::task::spawn_blocking(move || {
            db.write()
                .compact()
                .map_err(|e| StorageError::FlushError(e.to_string()))
        })
        .await
        .expect("Failed to flush database")
        {
            Ok(_) => Ok(()),
            Err(e) => Err(StorageError::FlushError(e.to_string())),
        }
    }

    async fn create_table(&mut self, table: String) -> Result<(), StorageError> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let table_def: TableDefinition<u64, &[u8]> = TableDefinition::new(&table);
            {
                let tx = db
                    .read()
                    .begin_write()
                    .map_err(|e| StorageError::TableError(e.to_string()))?;
                {
                    tx.open_table(table_def)
                        .map_err(|e| StorageError::TableError(e.to_string()))?;
                }
                tx.commit()
                    .map_err(|e| StorageError::CommitError(e.to_string()))?;
                Ok::<(), StorageError>(())
            }
        })
        .await
        .expect("Failed to create table")?;
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

    async fn setup_backend() -> RedbBackend {
        let db_file = tempdir().unwrap().into_path();
        let path = db_file.join("test.db");
        let mut backend = RedbBackend::initialize(Some(path)).await.unwrap();
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