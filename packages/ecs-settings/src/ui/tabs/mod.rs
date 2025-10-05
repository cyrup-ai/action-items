pub mod general;
pub mod extensions;
pub mod ai;
pub mod cloud_sync;
pub mod account;
pub mod organizations;
pub mod advanced;
pub mod about;

pub use general::create_general_tab;
pub use extensions::create_extensions_tab;
pub use ai::create_ai_tab;
pub use cloud_sync::create_cloud_sync_tab;
pub use account::create_account_tab;
pub use organizations::create_organizations_tab;
pub use advanced::create_advanced_tab;
pub use about::create_about_tab;
