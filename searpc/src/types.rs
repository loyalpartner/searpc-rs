use serde::Serialize;
use serde_json::Value;

/// RPC function argument types
///
/// This enum replaces the C version's string-based type checking.
/// Good taste: use type system instead of strcmp()
///
/// Note: All variants can be serialized as JSON null by wrapping in `Option<Arg>`.
/// The C version has add_string_or_null_element() - we achieve this via Option.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Arg {
    /// JSON null value (matches C's null handling)
    Null,
    /// 32-bit integer
    Int(i32),
    /// 64-bit integer
    Int64(i64),
    /// String (or null via `Option<Arg>`)
    String(String),
    /// Arbitrary JSON value (or null via `Option<Arg>`)
    Json(Value),
}

impl Arg {
    pub fn null() -> Self {
        Arg::Null
    }

    pub fn int(v: i32) -> Self {
        Arg::Int(v)
    }

    pub fn int64(v: i64) -> Self {
        Arg::Int64(v)
    }

    pub fn string(s: impl Into<String>) -> Self {
        Arg::String(s.into())
    }

    pub fn json(v: Value) -> Self {
        Arg::Json(v)
    }
}

// Convenience From implementations
impl From<i32> for Arg {
    fn from(v: i32) -> Self {
        Arg::Int(v)
    }
}

impl From<i64> for Arg {
    fn from(v: i64) -> Self {
        Arg::Int64(v)
    }
}

impl From<&str> for Arg {
    fn from(s: &str) -> Self {
        Arg::String(s.to_string())
    }
}

impl From<String> for Arg {
    fn from(s: String) -> Self {
        Arg::String(s)
    }
}

impl From<Value> for Arg {
    fn from(v: Value) -> Self {
        Arg::Json(v)
    }
}

/// Trait for types that can be converted into RPC arguments
///
/// This trait is used by the `#[rpc]` macro to automatically convert
/// function parameters into `Arg` enum variants.
pub trait IntoArg {
    fn into_arg(self) -> Arg;
}

impl IntoArg for i32 {
    fn into_arg(self) -> Arg {
        Arg::Int(self)
    }
}

impl IntoArg for i64 {
    fn into_arg(self) -> Arg {
        Arg::Int64(self)
    }
}

impl IntoArg for &str {
    fn into_arg(self) -> Arg {
        Arg::String(self.to_string())
    }
}

impl IntoArg for String {
    fn into_arg(self) -> Arg {
        Arg::String(self)
    }
}

impl IntoArg for Value {
    fn into_arg(self) -> Arg {
        Arg::Json(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_creation() {
        let _a1: Arg = 42.into();
        let _a2: Arg = "hello".into();
        let _a3 = Arg::int64(1234567890);
    }

    #[test]
    fn test_arg_serialization() {
        let args = vec![
            Arg::int(42),
            Arg::string("test"),
            Arg::int64(9999),
        ];

        let json = serde_json::to_string(&args).unwrap();
        assert_eq!(json, r#"[42,"test",9999]"#);
    }

    #[test]
    fn test_arg_null() {
        let args = vec![
            Arg::int(42),
            Arg::null(),
            Arg::string("test"),
        ];

        let json = serde_json::to_string(&args).unwrap();
        assert_eq!(json, r#"[42,null,"test"]"#);
    }
}
