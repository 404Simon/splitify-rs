//! Input validation and sanitization utilities
//!
//! This module provides centralized validation to ensure data integrity,
//! security, and consistency across the application.

pub mod auth;
pub mod financial;

#[cfg(feature = "ssr")]
pub use auth::*;
#[cfg(feature = "ssr")]
pub use financial::*;
use leptos::prelude::*;

/// Sanitize string by removing control characters and trimming whitespace
///
/// # Examples
/// ```
/// use rustify_app::validation::sanitize_string;
///
/// assert_eq!(sanitize_string("  hello  "), "hello");
/// assert_eq!(sanitize_string("hello\nworld"), "hello world");
/// ```
pub fn sanitize_string(input: &str) -> String {
    input
        .trim()
        .chars()
        .map(|c| {
            if c.is_control() {
                if c.is_whitespace() {
                    ' '
                } else {
                    '\0'
                }
            } else {
                c
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Validate and sanitize a name field (groups, debts, etc.)
///
/// # Examples
/// ```
/// use rustify_app::validation::validate_name;
///
/// assert!(validate_name("My Group", 1, 255, "Group name").is_ok());
/// assert!(validate_name("", 1, 255, "Group name").is_err());
/// ```
pub fn validate_name(
    input: &str,
    min_len: usize,
    max_len: usize,
    field_name: &str,
) -> Result<String, ServerFnError> {
    let sanitized = sanitize_string(input);

    if sanitized.is_empty() || sanitized.len() < min_len {
        return Err(ServerFnError::new(format!(
            "{} must be at least {} character{}",
            field_name,
            min_len,
            if min_len == 1 { "" } else { "s" }
        )));
    }

    if sanitized.len() > max_len {
        return Err(ServerFnError::new(format!(
            "{} must be {} characters or less",
            field_name, max_len
        )));
    }

    Ok(sanitized)
}

/// Validate a description field (allows empty, enforces max length)
#[cfg(feature = "ssr")]
pub fn validate_description(description: &str, max_len: usize) -> Result<String, ServerFnError> {
    let sanitized = sanitize_string(description);

    if sanitized.len() > max_len {
        return Err(ServerFnError::new(format!(
            "Description must be {} characters or less",
            max_len
        )));
    }

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("  hello  "), "hello");
        assert_eq!(sanitize_string("hello\nworld"), "hello world");
        assert_eq!(sanitize_string("test\x00string"), "teststring");
        assert_eq!(sanitize_string("\t  spaces  \n"), "spaces");
    }

    #[test]
    fn test_validate_name() {
        assert!(validate_name("My Group", 1, 255, "Group name").is_ok());
        assert!(validate_name("A", 1, 255, "Group name").is_ok());
        assert!(validate_name("", 1, 255, "Group name").is_err());
        assert!(validate_name(&"a".repeat(300), 1, 255, "Group name").is_err());
        assert!(validate_name("  Valid  ", 1, 255, "Group name").is_ok());
    }
}
