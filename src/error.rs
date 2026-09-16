use std::error::Error;
use std::fmt;
use serde_json::Value;
use crate::event::ErrorInfo;

#[derive(Debug, Clone)]
pub struct ParsedError {
    pub message: String,
    pub code: Option<String>,
    pub status: u16,
    pub why: Option<String>,
    pub fix: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug)]
pub struct EvlogError {
    pub message: String,
    pub code: Option<String>,
    pub status: u16,
    pub why: Option<String>,
    pub fix: Option<String>,
    pub link: Option<String>,
    pub internal: Option<Value>,
    source: Option<String>,
    location: String,
    stack: String,
}

impl EvlogError {
    // `#[track_caller]` allows us to grab the exact file and line number where 
    // `EvlogError::new` is invoked, rather than the line number *inside* the macro 
    // or this function. This is critical for debugging domain errors.
    #[track_caller]
    pub fn new(message: impl Into<String>) -> Self {
        let location = std::panic::Location::caller();
        let loc_str = format!("{}:{}", location.file(), location.line());
        
        // Capture the call stack so developers can see the exact execution path 
        // leading to the error. This is not sent in the HTTP response.
        let backtrace = std::backtrace::Backtrace::capture();
        
        Self {
            message: message.into(),
            code: None, status: 500, why: None, fix: None, link: None, internal: None, source: None,
            location: loc_str,
            stack: backtrace.to_string(),
        }
    }
    pub fn code(mut self, code: impl Into<String>) -> Self { self.code = Some(code.into()); self }
    pub fn status(mut self, status: u16) -> Self { self.status = status; self }
    pub fn why(mut self, why: impl Into<String>) -> Self { self.why = Some(why.into()); self }
    pub fn fix(mut self, fix: impl Into<String>) -> Self { self.fix = Some(fix.into()); self }
    pub fn link(mut self, link: impl Into<String>) -> Self { self.link = Some(link.into()); self }
    pub fn internal(mut self, internal: Value) -> Self { self.internal = Some(internal); self }
    pub fn cause(mut self, cause: &dyn Error) -> Self { self.source = Some(cause.to_string()); self }

    pub fn to_error_info(&self) -> ErrorInfo {
        let mut internal_map = self.internal.clone().unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = internal_map.as_object_mut() {
            obj.insert("location".to_string(), serde_json::json!(self.location));
        } else {
            internal_map = serde_json::json!({ "value": internal_map, "location": self.location });
        }

        ErrorInfo {
            name: "EvlogError".to_string(),
            message: self.message.clone(),
            code: self.code.clone(),
            why: self.why.clone(),
            fix: self.fix.clone(),
            link: self.link.clone(),
            stack: Some(self.stack.clone()),
            internal: Some(internal_map),
        }
    }
}

impl fmt::Display for EvlogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.message) }
}
impl Error for EvlogError {}

impl serde::Serialize for EvlogError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut count = 1; // message always present
        if self.code.is_some() { count += 1; }
        if self.status != 0 { count += 1; }
        if self.why.is_some() { count += 1; }
        if self.fix.is_some() { count += 1; }
        if self.link.is_some() { count += 1; }
        let mut map = serializer.serialize_map(Some(count))?;
        map.serialize_entry("message", &self.message)?;
        if let Some(ref code) = self.code {
            map.serialize_entry("code", code)?;
        }
        if self.status != 0 {
            map.serialize_entry("status", &self.status)?;
        }
        if let Some(ref why) = self.why {
            map.serialize_entry("why", why)?;
        }
        if let Some(ref fix) = self.fix {
            map.serialize_entry("fix", fix)?;
        }
        if let Some(ref link) = self.link {
            map.serialize_entry("link", link)?;
        }
        map.end()
    }
}

impl EvlogError {
    /// Serialize to JSON string for HTTP error responses.
    /// Only user-facing fields (no stack, no location, no internal).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            format!("{{\"message\":\"{}\"}}", self.message)
        })
    }

    /// Build the full response body: `{"error": {...}}`
    pub fn to_response_body(&self) -> String {
        format!("{{\"error\":{}}}", self.to_json())
    }
}

#[macro_export]
macro_rules! create_error {
    (
        message: $message:expr
        $(, code: $code:expr)? $(, status: $status:expr)? $(, why: $why:expr)?
        $(, fix: $fix:expr)? $(, link: $link:expr)? $(, internal: $internal:expr)? $(, cause: $cause:expr)? $(,)?
    ) => {{
        let mut err = $crate::error::EvlogError::new($message);
        $( err = err.code($code); )? $( err = err.status($status); )? $( err = err.why($why); )?
        $( err = err.fix($fix); )? $( err = err.link($link); )? $( err = err.internal($internal); )? $( err = err.cause($cause); )?
        err
    }};
}

/// Define an Error Catalog statically.
///
/// # Example
/// ```rust
/// use evlog::define_error_catalog;
/// 
/// define_error_catalog! {
///     pub enum ApiError {
///         BadRequest => {
///             message: "Bad request payload",
///             code: "BAD_REQ",
///             status: 400,
///             why: "The JSON could not be parsed.",
///             fix: "Check your syntax."
///         },
///         NotFound(id: String) => {
///             message: format!("Resource {} not found", id),
///             code: "NOT_FOUND",
///             status: 404
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_error_catalog {
    (
        $vis:vis enum $name:ident {
            $(
                $variant:ident $( ( $($field:ident : $ty:ty),* ) )? => {
                    message: $msg:expr
                    $(, code: $code:expr)?
                    $(, status: $status:expr)?
                    $(, why: $why:expr)?
                    $(, fix: $fix:expr)?
                    $(, link: $link:expr)?
                    $(,)?
                }
            ),* $(,)?
        }
    ) => {
        $vis enum $name {
            $(
                $variant $( ( $($ty),* ) )?
            ),*
        }

        impl $name {
            #[track_caller]
            pub fn into_evlog(&self) -> $crate::error::EvlogError {
                match self {
                    $(
                        Self::$variant $( ( $($field),* ) )? => {
                            $( $( let _ = $field; )* )?
                            $crate::create_error!(
                                message: $msg
                                $(, code: $code)?
                                $(, status: $status)?
                                $(, why: $why)?
                                $(, fix: $fix)?
                                $(, link: $link)?
                            )
                        }
                    ),*
                }
            }

            pub const SCHEMA_MD: &'static str = concat!(
                "## ", stringify!($name), "\n",
                $(
                    "- `", stringify!($variant), "`\n"
                ),*
            );
        }
    };
}

/// Dumps the markdown schema of the specified error catalogs into a file.
/// This creates a small `#[test]` function that writes the file when you run `cargo test`.
///
/// Example: `evlog::dump_schemas!(".ai/evlog_schema.md", ApiError, DatabaseError);`
#[macro_export]
macro_rules! dump_schemas {
    ($path:expr, $($name:ty),+) => {
        #[cfg(test)]
        #[test]
        fn __evlog_dump_schemas() {
            let mut out = String::new();
            out.push_str("# Evlog Error Catalog\n\n");
            out.push_str("> This file is auto-generated by `evlog::dump_schemas!`. DO NOT EDIT.\n");
            out.push_str("> AI Instructions: Use these errors when throwing domain errors instead of creating new ones.\n\n");
            $(
                out.push_str(<$name>::SCHEMA_MD);
                out.push_str("\n");
            )+
            
            if let Some(parent) = std::path::Path::new($path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::write($path, out).expect("Failed to write evlog schemas");
        }
    };
}
