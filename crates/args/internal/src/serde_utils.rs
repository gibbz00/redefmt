pub fn is_false(bool: &bool) -> bool {
    !(*bool)
}

/// Use only then T::default() is cheap
pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[cfg(test)]
pub mod assert {
    use serde::{Deserialize, Serialize};

    pub fn borrowed_bijective_serialization<'a, T: Serialize + Deserialize<'a> + core::fmt::Debug + PartialEq>(
        json_str: &'a str,
        t: &T,
    ) {
        serialize(t, json_str);
        deserialize(json_str, t);
    }

    pub fn serialize<T: Serialize + core::fmt::Debug + PartialEq>(initial: &T, expected_json_str: &str) {
        let actual_json_str = serde_json::to_string(initial).unwrap();
        assert_eq!(expected_json_str, actual_json_str)
    }

    pub fn deserialize<'a, T: Deserialize<'a> + core::fmt::Debug + PartialEq>(json_str: &'a str, expected: &T) {
        let actual = serde_json::from_str(json_str).unwrap();
        assert_eq!(expected, &actual)
    }
}
