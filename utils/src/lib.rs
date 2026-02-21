mod deserialize;
mod discord;
mod legacy;
mod logging;
mod pagination;
mod parser;
mod permissions;

use std::{fmt::Debug, ops::Deref, sync::Arc};

pub use deserialize::*;
pub use discord::*;
pub use legacy::*;
pub use pagination::*;
pub use parser::*;
pub use permissions::*;
use serenity::{
    all::{Member, Timestamp},
    prelude::TypeMapKey,
};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct Pointer<T: ?Sized>(Arc<RwLock<T>>)
where
    T: Send + Sync + 'static;

impl<T> Pointer<T>
where
    T: Send + Sync + Sized + 'static,
{
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

    pub fn inner(self) -> Result<T, String> {
        let value = Arc::try_unwrap(self.0).map_err(|_| "Failed to unwrap Arc".to_string())?;
        Ok(value.into_inner())
    }

    pub async fn set(&self, value: T) -> Self {
        let mut write = self.0.write().await;
        *write = value;
        self.clone()
    }
}

impl<T: Clone> Pointer<T>
where
    T: Send + Sync + Sized + 'static,
{
    pub async fn make_clone(&self) -> T {
        self.0.read().await.clone()
    }
}

impl<T> Clone for Pointer<T>
where
    T: Send + Sync + Sized + 'static,
{
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> From<T> for Pointer<T>
where
    T: Send + Sync + Sized + 'static,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for Pointer<T>
where
    T: Send + Sync + Sized + 'static,
{
    type Target = Arc<RwLock<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Default> Default for Pointer<T>
where
    T: Send + Sync + Sized + 'static,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Send + Sync + 'static> TypeMapKey for Pointer<T> {
    type Value = Pointer<T>;
}

// impl<T: Debug> std::fmt::Debug for Pointer<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Pointer {{ {:?} }}", self.0)
//     }
// }

impl<T> std::fmt::Debug for Pointer<T>
where
    T: Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pointer < {:?} >", std::any::type_name::<T>())
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

pub type HttpType = Arc<serenity::http::Http>;
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
            ResponseError::Err(msg) => write!(f, "{}", msg),
            ResponseError::Warn(msg) => write!(f, "{}", msg),
            ResponseError::Info(msg) => write!(f, "{}", msg),
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
