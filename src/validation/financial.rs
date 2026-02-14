//! Financial validation (amounts and currency)

#[cfg(feature = "ssr")]
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
use super::sanitize_string;

/// Validate amount (decimal string) for financial transactions
///
/// Ensures amounts are:
/// - Valid decimal numbers
/// - Greater than zero
/// - Have at most 2 decimal places
/// - Not excessively large (max 999,999,999.99)
///
/// # Examples
/// ```
/// use rustify_app::validation::validate_amount;
///
/// assert!(validate_amount("10.50").is_ok());
/// assert!(validate_amount("100").is_ok());
/// assert!(validate_amount("0").is_err()); // Must be > 0
/// assert!(validate_amount("10.999").is_err()); // Too many decimals
/// ```
#[cfg(feature = "ssr")]
pub fn validate_amount(amount: &str) -> Result<Decimal, ServerFnError> {
    let sanitized = sanitize_string(amount);

    if sanitized.is_empty() {
        return Err(ServerFnError::new("Amount is required"));
    }

    let amount_decimal = sanitized.parse::<Decimal>().map_err(|_| {
        ServerFnError::new(
            "Invalid amount format. Please use numbers and a decimal point (e.g., 10.50)",
        )
    })?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new("Amount must be greater than zero"));
    }

    if amount_decimal.scale() > 2 {
        return Err(ServerFnError::new(
            "Amount can have at most 2 decimal places",
        ));
    }

    // Prevent overflow
    if amount_decimal > Decimal::from(999_999_999) {
        return Err(ServerFnError::new(
            "Amount is too large. Maximum is 999,999,999.99",
        ));
    }

    Ok(amount_decimal)
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn test_validate_amount() {
        assert_eq!(validate_amount("10.50").unwrap(), Decimal::new(1050, 2));
        assert_eq!(validate_amount("100").unwrap(), Decimal::from(100));
        assert_eq!(validate_amount("0.01").unwrap(), Decimal::new(1, 2));

        assert!(validate_amount("0").is_err()); // Zero
        assert!(validate_amount("-5").is_err()); // Negative
        assert!(validate_amount("10.999").is_err()); // Too many decimals
        assert!(validate_amount("abc").is_err()); // Invalid format
        assert!(validate_amount("").is_err()); // Empty
        assert!(validate_amount("1000000000").is_err()); // Too large
    }
}
