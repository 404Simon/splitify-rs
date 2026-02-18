pub mod groups;
pub mod home;
pub mod invite_accept;
pub mod login;
pub mod offline;
pub mod recurring_debts;
pub mod register;
pub mod shared_debts;
pub mod shopping_lists;
pub mod transactions;

// Re-export page components
pub use groups::{GroupsCreate, GroupsEdit, GroupsIndex, GroupsInvites, GroupsShow};
pub use home::HomePage;
pub use invite_accept::InviteAccept;
pub use login::LoginPage;
pub use offline::OfflinePage;
pub use recurring_debts::{RecurringDebtsCreate, RecurringDebtsEdit, RecurringDebtsShow};
pub use register::RegisterPage;
pub use shared_debts::{SharedDebtsCreate, SharedDebtsEdit};
pub use shopping_lists::{ShoppingListCreate, ShoppingListEdit, ShoppingListShow};
pub use transactions::{TransactionsCreate, TransactionsEdit};
