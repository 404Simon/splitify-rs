use rust_decimal::Decimal;

use super::models::UserShare;

/// Calculate individual shares for a shared debt
/// Divides the total amount equally among all participants
pub fn calculate_shares(amount: Decimal, user_ids: &[(i64, String)]) -> Vec<UserShare> {
    if user_ids.is_empty() {
        return Vec::new();
    }

    let count = Decimal::from(user_ids.len());
    let share_per_user = (amount / count).round_dp(2); // Round to 2 decimal places

    user_ids
        .iter()
        .map(|(user_id, username)| UserShare {
            user_id: *user_id,
            username: username.clone(),
            share_amount: share_per_user,
        })
        .collect()
}
