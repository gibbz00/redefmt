pub fn is_false(bool: &bool) -> bool {
    !(*bool)
}

/// Use only then T::default() is cheap
pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[cfg(all(test, feature = "serde"))]
pub mod assert {
    use serde::{Serialize, de::DeserializeOwned};

    pub fn bijective_serialization<T: Serialize + DeserializeOwned + core::fmt::Debug + PartialEq>(initial: T) {
        let json_string = serde_json::to_string(&initial).unwrap();
        let r#final = serde_json::from_str(&json_string).unwrap();
        assert_eq!(initial, r#final)
    }
}
