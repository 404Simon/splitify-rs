pub mod groups;
pub mod home;
pub mod invite_accept;
pub mod login;
pub mod register;

// Re-export page components
pub use groups::{GroupsCreate, GroupsEdit, GroupsIndex, GroupsInvites, GroupsShow};
pub use home::HomePage;
pub use invite_accept::InviteAccept;
pub use login::LoginPage;
pub use register::RegisterPage;
