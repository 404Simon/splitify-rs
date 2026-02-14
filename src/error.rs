use leptos::prelude::ServerFnError;

/// Custom application error type with better conversion handling
#[cfg(feature = "ssr")]
#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Authentication(String),
    Authorization(String),
    Validation(String),
    NotFound(String),
    Internal(String),
}

#[cfg(feature = "ssr")]
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            Self::Authorization(msg) => write!(f, "Authorization error: {}", msg),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

#[cfg(feature = "ssr")]
impl std::error::Error for AppError {}

#[cfg(feature = "ssr")]
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

#[cfg(feature = "ssr")]
impl AppError {
    /// Convert to ServerFnError
    pub fn into_server_error(self) -> ServerFnError {
        // Convert AppError to ServerFnError
        // For production, you might want to log detailed errors but return generic messages
        match self {
            Self::Database(e) => {
                // Log the actual database error
                eprintln!("Database error: {:?}", e);
                ServerFnError::new("A database error occurred")
            }
            Self::Authentication(msg) => ServerFnError::new(msg),
            Self::Authorization(msg) => ServerFnError::new(msg),
            Self::Validation(msg) => ServerFnError::new(msg),
            Self::NotFound(msg) => ServerFnError::new(msg),
            Self::Internal(e) => {
                eprintln!("Internal error: {}", e);
                ServerFnError::new("An internal error occurred")
            }
        }
    }

    /// Create authentication error
    pub fn auth<S: Into<String>>(msg: S) -> Self {
        Self::Authentication(msg.into())
    }

    /// Create authorization error
    pub fn unauthorized<S: Into<String>>(msg: S) -> Self {
        Self::Authorization(msg.into())
    }

    /// Create validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Create not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}
