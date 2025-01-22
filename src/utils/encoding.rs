use serde::{Serialize, Deserialize};
use bincode;

pub fn serialize<T: Serialize>(data: &T) -> Result<Vec<u8>, String> {
    bincode::serialize(data).map_err(|e| e.to_string())
}

pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T, String> {
    bincode::deserialize(data).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        id: u32,
        name: String,
    }

    #[test]
    fn test_serialize_deserialize() {
        let original = TestStruct {
            id: 42,
            name: "Test".to_string(),
        };

        let serialized = serialize(&original).unwrap();
        let deserialized: TestStruct = deserialize(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}