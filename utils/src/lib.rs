mod discord;
mod legacy;
mod logging;
mod pagination;
mod parser;
mod permissions;

use std::{ops::Deref, sync::Arc};

pub use discord::*;
pub use legacy::*;
pub use logging::*;
pub use pagination::*;
pub use parser::*;
pub use permissions::*;
use serenity::all::{Member, Timestamp};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Default, Debug)]
pub struct Pointer<T>(Arc<RwLock<T>>);

impl<T> Pointer<T> {
    pub fn new(inner: T) -> Self {
        Self(Arc::new(RwLock::new(inner)))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().await
    }

    pub fn read_sync(&self) -> RwLockReadGuard<'_, T> {
        self.0.blocking_read()
    }

    pub fn write_sync(&self) -> RwLockWriteGuard<'_, T> {
        self.0.blocking_write()
    }

    pub fn inner(self) -> T {
        Arc::try_unwrap(self.0)
            .ok()
            .expect("Multiple pointers exist")
            .into_inner()
    }
}

impl<T: Clone> Pointer<T> {
    pub async fn make_clone(&self) -> T {
        self.0.read().await.clone()
    }
}

impl<T> Clone for Pointer<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[derive(Debug)]
pub struct ElapsedTime {
    start: std::time::Instant,
}

impl Default for ElapsedTime {
    fn default() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl ElapsedTime {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
    pub fn elapsed_s(&self) -> u64 {
        self.start.elapsed().as_secs()
    }

    pub fn reset(&mut self) {
        self.start = std::time::Instant::now();
    }
    pub fn reset_and_get(&mut self) -> std::time::Duration {
        let elapsed = self.start.elapsed();
        self.reset();
        elapsed
    }
}

pub type Http = Arc<serenity::http::Http>;
pub type DataType = Arc<RwLock<serenity::prelude::TypeMap>>;

pub enum ResponseError {
    Err(String),
    Warn(String),
    Info(String),
}
impl ResponseError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Err(msg.into())
    }

    pub fn warn(msg: impl Into<String>) -> Self {
        Self::Warn(msg.into())
    }

    pub fn info(msg: impl Into<String>) -> Self {
        Self::Info(msg.into())
    }
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::Err(msg) => write!(f, "Error: {}", msg),
            ResponseError::Warn(msg) => write!(f, "Warning: {}", msg),
            ResponseError::Info(msg) => write!(f, "Info: {}", msg),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemberData {
    pub is_bot: bool,
    pub join_date: Timestamp,
}

impl MemberData {
    pub fn new(member: &Member) -> Self {
        Self {
            is_bot: member.user.bot,
            join_date: member.joined_at.unwrap_or_else(Timestamp::now), // Fallback to now if joined_at is None
        }
    }
}
