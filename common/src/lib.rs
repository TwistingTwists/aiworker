pub mod error;
pub mod util;
pub mod download;

pub mod client;
pub use client::Client;

pub mod config;
pub use config::Config;

pub mod types;

pub mod chat;
pub use chat::Chat;

// pub mod model;
// pub use model::Model;