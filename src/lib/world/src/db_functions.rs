use ferrumc_storage::compressors::Compressor;
use crate::chunk_format::Chunk;
use crate::errors::WorldError;
use crate::World;

impl World {
    // Store a chunk in the database. Internally, this function serializes the chunk to a byte array,
    // compresses it, and stores it in the database.
    pub async fn save_chunk(&self, chunk: Chunk) -> Result<(), WorldError> {
        save_chunk_internal(self, chunk).await
    }
    
    // Load a chunk from the database. Internally, this function retrieves the chunk from the database,
    // decompresses it, and deserializes it.
    pub async fn load_chunk(&self, x: i32, z: i32) -> Result<Chunk, WorldError> {
        load_chunk_internal(self, &self.compressor, x, z).await
    }
    
    // Check if a chunk exists in the database.
    pub async fn chunk_exists(&self, x: i32, z: i32) -> Result<bool, WorldError> {
        chunk_exists_internal(self, x, z).await
    }
    
    // Delete a chunk from the database.
    pub async fn delete_chunk(&self, x: i32, z: i32) -> Result<(), WorldError> {
        delete_chunk_internal(self, x, z).await
    }
    
    // Sync the database. This function flushes the database, ensuring that all data is written to disk.
    // This can be a no-op for some databases.
    pub async fn sync(&self) -> Result<(), WorldError> {
        sync_internal(self).await
    }
}

// Internal functions for interacting with the database. These don't implement caching in any way and
// have a strange interface because they are meant to be used internally by the World struct. They 
// can be useful if you are struggling with the borrow checker or if you want to implement your own
// caching layer, but generally you shouldn't use these functions directly.

/// Save a chunk to the database without caching.
pub(crate) async fn save_chunk_internal(
    world: &World,
    chunk: Chunk,
) -> Result<(), WorldError> {
    let as_bytes = world.compressor.compress(&bitcode::encode(&chunk))?;
    let digest = ferrumc_general_purpose::hashing::hash((chunk.x, chunk.z));
    world.storage_backend.upsert("chunks".to_string(), digest, as_bytes).await?;
    Ok(())
}

/// Load a chunk from the database without caching.
pub(crate) async fn load_chunk_internal(
    world: &World,
    compressor: &Compressor,
    x: i32,
    z: i32,
) -> Result<Chunk, WorldError> {
    let digest = ferrumc_general_purpose::hashing::hash((x, z));
    match world.storage_backend.get("chunks".to_string(), digest).await? {
        Some(compressed) => {
            let data = compressor.decompress(&compressed)?;
            let chunk: Chunk = bitcode::decode(&data).map_err(|e| WorldError::BitcodeDecodeError(e.to_string()))?;
            Ok(chunk)
        }
        None => Err(WorldError::ChunkNotFound),
    }
}

/// Check if a chunk exists in the database without caching.
pub(crate) async fn chunk_exists_internal(
    world: &World,
    x: i32,
    z: i32,
) -> Result<bool, WorldError> {
    let digest = ferrumc_general_purpose::hashing::hash((x, z));
    Ok(world.storage_backend.exists("chunks".to_string(), digest).await?)
}

/// Delete a chunk from the database.
pub(crate) async fn delete_chunk_internal(
    world: &World,
    x: i32,
    z: i32,
) -> Result<(), WorldError> {
    let digest = ferrumc_general_purpose::hashing::hash((x, z));
    world.storage_backend.delete("chunks".to_string(), digest).await?;
    Ok(())
}

/// Sync the database.
pub(crate) async fn sync_internal(world: &World) -> Result<(), WorldError> {
    world.storage_backend.flush().await?;
    Ok(())
}