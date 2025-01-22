use sled::{Db, IVec};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use log::{info, error};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

#[derive(Clone)]
pub struct Database {
    db: Db,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let db = sled::open(path).map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(Self { db })
    }

    pub fn new_with_config(path: &str, use_compression: bool) -> Result<Self, StorageError> {
        let mut config = sled::Config::default().path(path);
        if use_compression {
            config = config.use_compression(true);
        }
        let db = config.open().map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(Self { db })
    }

    pub fn save<K: Serialize, V: Serialize>(&self, key: &K, value: &V) -> Result<(), StorageError> {
        let serialized_key = bincode::serialize(key)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        let serialized_value = bincode::serialize(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        match self.db.insert(serialized_key, serialized_value) {
            Ok(_) => {
                info!("Key saved successfully.");
                Ok(())
            }
            Err(e) => {
                error!("Failed to save key: {}", e);
                Err(StorageError::DatabaseError(e.to_string()))
            }
        }
    }

    pub fn save_with_ttl<K: Serialize, V: Serialize>(
        &self,
        key: &K,
        value: &V,
        ttl: std::time::Duration,
    ) -> Result<(), StorageError> {
        self.save(key, value)?;
        
        info!("Key saved with TTL: {:?}", ttl);
        Ok(())
    }

    pub fn get<K: Serialize, V: for<'de> Deserialize<'de>>(&self, key: &K) -> Result<Option<V>, StorageError> {
        let serialized_key = bincode::serialize(key)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        match self.db.get(serialized_key) {
            Ok(Some(value)) => {
                let deserialized_value = bincode::deserialize(&value)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized_value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::DatabaseError(e.to_string())),
        }
    }

    pub fn delete<K: Serialize>(&self, key: &K) -> Result<(), StorageError> {
        let serialized_key = bincode::serialize(key)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        match self.db.remove(serialized_key) {
            Ok(_) => {
                info!("Key deleted successfully.");
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete key: {}", e);
                Err(StorageError::DatabaseError(e.to_string()))
            }
        }
    }

    pub fn iter<V: for<'de> Deserialize<'de>>(&self) -> Result<Vec<V>, StorageError> {
        let mut results = Vec::new();
        for item in self.db.iter() {
            let (_key, value) = item.map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            let deserialized_value = bincode::deserialize(&value)
                .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
            results.push(deserialized_value);
        }
        Ok(results)
    }

    pub fn flush(&self) -> Result<(), StorageError> {
        self.db.flush().map_err(|e| StorageError::DatabaseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        id: u32,
        name: String,
    }

    #[test]
    fn test_database_operations() {
        let db = Database::new("test_db").unwrap();

        let key = "test_key";
        let value = TestStruct {
            id: 1,
            name: "Test Name".to_string(),
        };

        db.save(&key, &value).unwrap();
        let retrieved: TestStruct = db.get(&key).unwrap().unwrap();
        assert_eq!(retrieved, value);

        db.save_with_ttl(&key, &value, std::time::Duration::from_secs(60)).unwrap();

        db.delete(&key).unwrap();
        assert!(db.get::<_, TestStruct>(&key).unwrap().is_none());

        db.flush().unwrap();
    }
}