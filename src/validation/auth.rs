//! Authentication-related validation (username, password, email)

#[cfg(feature = "ssr")]
use std::sync::OnceLock;

#[cfg(feature = "ssr")]
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use regex::Regex;

#[cfg(feature = "ssr")]
use super::sanitize_string;

/// Validate username (alphanumeric, underscore, hyphen only, 3-50 chars)
///
/// # Examples
/// ```
/// use rustify_app::validation::validate_username;
///
/// assert!(validate_username("john_doe").is_ok());
/// assert!(validate_username("user-123").is_ok());
/// assert!(validate_username("ab").is_err()); // Too short
/// ```
#[cfg(feature = "ssr")]
pub fn validate_username(username: &str) -> Result<String, ServerFnError> {
    let sanitized = sanitize_string(username);

    if sanitized.is_empty() {
        return Err(ServerFnError::new("Username is required"));
    }

    if sanitized.len() < 3 {
        return Err(ServerFnError::new("Username must be at least 3 characters"));
    }

    if sanitized.len() > 50 {
        return Err(ServerFnError::new("Username must be 50 characters or less"));
    }

    if !sanitized
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ServerFnError::new(
            "Username can only contain letters, numbers, underscores, and hyphens",
        ));
    }

    Ok(sanitized)
}

/// Validate password strength (min 8 chars, max 128, must have letter)
#[cfg(feature = "ssr")]
pub fn validate_password(password: &str) -> Result<(), ServerFnError> {
    if password.is_empty() {
        return Err(ServerFnError::new("Password is required"));
    }

    if password.len() < 8 {
        return Err(ServerFnError::new("Password must be at least 8 characters"));
    }

    if password.len() > 128 {
        return Err(ServerFnError::new(
            "Password must be 128 characters or less",
        ));
    }

    if !password.chars().any(|c| c.is_alphabetic()) {
        return Err(ServerFnError::new(
            "Password must contain at least one letter",
        ));
    }

    Ok(())
}

/// Check if email format is valid (RFC 5322 compliant)
///
/// # Examples
/// ```
/// use rustify_app::validation::is_valid_email;
///
/// assert!(is_valid_email("user@example.com"));
/// assert!(is_valid_email("user.name+tag@example.co.uk"));
/// assert!(!is_valid_email("a@b.c")); // Domain too short
/// assert!(!is_valid_email("@example.com")); // No local part
/// ```
#[cfg(feature = "ssr")]
pub fn is_valid_email(email: &str) -> bool {
    static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

    let regex = EMAIL_REGEX.get_or_init(|| {
        Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
        ).expect("Failed to compile email regex")
    });

    // Check format and length (RFC 5321: max 254 chars)
    if email.len() > 254 || !regex.is_match(email) {
        return false;
    }

    // Ensure TLD is at least 2 chars
    if let Some(domain) = email.split('@').nth(1) {
        if let Some(tld) = domain.split('.').next_back() {
            return tld.len() >= 2;
        }
    }

    false
}

/// Validate email and return sanitized lowercase version
#[cfg(feature = "ssr")]
pub fn validate_email(email: &str) -> Result<String, ServerFnError> {
    let sanitized = sanitize_string(email).to_lowercase();

    if sanitized.is_empty() {
        return Err(ServerFnError::new("Email is required"));
    }

    if !is_valid_email(&sanitized) {
        return Err(ServerFnError::new(
            "Invalid email format. Please enter a valid email address (e.g., user@example.com)",
        ));
    }

    Ok(sanitized)
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(validate_username("john_doe").is_ok());
        assert!(validate_username("user-123").is_ok());
        assert!(validate_username("User123").is_ok());
        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username(&"a".repeat(51)).is_err()); // Too long
        assert!(validate_username("user@email").is_err()); // Invalid chars
        assert!(validate_username("user name").is_err()); // Space not allowed
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("password123").is_ok());
        assert!(validate_password("Pass1234").is_ok());
        assert!(validate_password("12345abcd").is_ok());
        assert!(validate_password("noNumbersHere").is_ok()); // Letters only is fine
        assert!(validate_password("short1").is_err()); // Too short
        assert!(validate_password("12345678").is_err()); // No letters
        assert!(validate_password("").is_err()); // Empty
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user.name+tag@example.co.uk"));
        assert!(is_valid_email("user_name@example-domain.com"));
        assert!(is_valid_email("user123@test.org"));

        assert!(!is_valid_email(""));
        assert!(!is_valid_email("a@b.c")); // TLD too short
        assert!(!is_valid_email("@example.com")); // No local part
        assert!(!is_valid_email("user@")); // No domain
        assert!(!is_valid_email("user example@test.com")); // Space
        assert!(!is_valid_email("user@.com")); // Domain starts with dot
        assert!(!is_valid_email("user@@example.com")); // Double @

        let long_email = format!("{}@example.com", "a".repeat(250));
        assert!(!is_valid_email(&long_email));
    }

    #[test]
    fn test_validate_email() {
        assert_eq!(
            validate_email("User@Example.COM").unwrap(),
            "user@example.com"
        );
        assert_eq!(
            validate_email("  user@test.com  ").unwrap(),
            "user@test.com"
        );
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid").is_err());
    }
}
