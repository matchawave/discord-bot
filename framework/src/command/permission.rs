use serenity::{
    all::{ChannelId, Context, GuildId, RoleId, UserId},
    async_trait,
};
use utils::{BotPermission, Parser, Pointer, ResponseError};

use crate::{
    extractors::{EventExtractor, Extractor},
    handler::DynCallback,
};

type PermissionResult = Result<bool, ResponseError>;

pub struct PermissionRequest {
    pub guild_id: GuildId,
    pub channel_id: ChannelId, // Optional channel ID for more granular permission checks
    pub identification: PermissionCheck,
    pub permissions: Vec<BotPermission>,
}

impl PermissionRequest {
    pub fn new_user(
        guild_id: GuildId,
        channel_id: ChannelId,
        user_id: UserId,
        permissions: &[BotPermission],
    ) -> Self {
        Self {
            guild_id,
            channel_id,
            identification: PermissionCheck::User(user_id),
            permissions: permissions.to_vec(),
        }
    }

    pub fn new_roles(
        guild_id: GuildId,
        channel_id: ChannelId,
        role_ids: &[RoleId],
        permissions: &[BotPermission],
    ) -> Self {
        Self {
            guild_id,
            channel_id,
            identification: PermissionCheck::Roles(role_ids.to_vec()),
            permissions: permissions.to_vec(),
        }
    }
}

pub enum PermissionCheck {
    Roles(Vec<RoleId>),
    User(UserId),
}

pub type PermissionCallback =
    Box<dyn DynCallback<PermissionRequest, PermissionResult> + Send + Sync>;

#[async_trait]
impl EventExtractor<PermissionRequest> for Vec<BotPermission> {
    async fn extract_event(request: &PermissionRequest) -> Option<Self> {
        Some(request.permissions.clone())
    }
}

#[async_trait]
impl<T> Extractor<T> for Vec<BotPermission>
where
    T: Send + Sync + 'static,
    Self: EventExtractor<T>,
{
    async fn extract(_ctx: &Context, ev: &T, _p: &Pointer<Parser>) -> Option<Self> {
        Self::extract_event(ev).await
    }
}
