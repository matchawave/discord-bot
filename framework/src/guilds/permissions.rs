use std::collections::HashMap;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{Context, GuildId, Member, RoleId, UserId},
    async_trait,
    prelude::TypeMapKey,
};
use utils::{BotPermission, Pointer};

use crate::{
    ShardData,
    extractors::{ContextEventExtractor, EventExtractor, Extractor},
};

pub struct FakePerms(Pointer<HashMap<BotPermission, Pointer<FakePermConfig>>>);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FakePermConfig {
    pub roles: Vec<RoleId>,
    pub users: Vec<UserId>,
}

impl TypeMapKey for FakePerms {
    type Value = Pointer<HashMap<BotPermission, Pointer<FakePermConfig>>>;
}

impl Default for FakePerms {
    fn default() -> Self {
        let perms = BotPermission::list();
        let mut map = HashMap::with_capacity(BotPermission::count());
        for perm in perms {
            map.insert(perm, FakePermConfig::default().into());
        }
        FakePerms(Pointer::new(map))
    }
}

impl FakePerms {
    pub fn new(map: &Pointer<HashMap<BotPermission, Pointer<FakePermConfig>>>) -> Self {
        FakePerms(map.clone())
    }

    pub async fn member_has_permission(
        &self,
        member: &Member,
        permissions: &[BotPermission],
    ) -> bool {
        let read = self.0.read().await;
        for permission in permissions {
            if let Some(perm_config_ptr) = read.get(&permission) {
                let perm_config = perm_config_ptr.read().await;
                if perm_config.users.par_iter().any(|&id| id == member.user.id) {
                    return true;
                }
                for role in &perm_config.roles {
                    if member.roles.par_iter().any(|r| r == role) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub async fn member_lacks_permission(
        &self,
        member: &Member,
        permissions: &[BotPermission],
    ) -> Option<Vec<BotPermission>> {
        let mut missing_perms = Vec::new();
        let read = self.0.read().await;
        for permission in permissions {
            if let Some(perm_config_ptr) = read.get(&permission) {
                let perm_config = perm_config_ptr.read().await;
                if perm_config.users.par_iter().any(|&id| id == member.user.id) {
                    continue;
                }
                let mut has_role = false;
                for role in &perm_config.roles {
                    if member.roles.par_iter().any(|r| r == role) {
                        has_role = true;
                        break;
                    }
                }
                if !has_role {
                    missing_perms.push(*permission);
                }
            } else {
                missing_perms.push(*permission);
            }
        }
        if missing_perms.is_empty() {
            None
        } else {
            Some(missing_perms)
        }
    }

    pub async fn set_permission_config(&self, permission: BotPermission, config: FakePermConfig) {
        let mut write = self.0.write().await;
        write.insert(permission, Pointer::new(config));
    }

    pub async fn add_user_to_permission(
        &self,
        permission: BotPermission,
        user_id: UserId,
    ) -> Result<(), String> {
        let read = self.0.read().await;
        if let Some(perm_config_ptr) = read.get(&permission) {
            let mut perm_config = perm_config_ptr.write().await;
            if !perm_config.users.par_iter().any(|&id| id == user_id) {
                perm_config.users.push(user_id);
                return Ok(());
            }
            return Err("User already has this permission".to_string());
        }
        Err(format!(
            "Permission config not found for permission: {:?}",
            permission
        ))
    }

    pub async fn add_role_to_permission(
        &self,
        permission: BotPermission,
        role_id: RoleId,
    ) -> Result<(), String> {
        let read = self.0.read().await;
        if let Some(perm_config_ptr) = read.get(&permission) {
            let mut perm_config = perm_config_ptr.write().await;
            if !perm_config.roles.par_iter().any(|r| *r == role_id) {
                perm_config.roles.push(role_id);
                return Ok(());
            }
            return Err("Role already has this permission".to_string());
        }
        Err(format!(
            "Permission config not found for permission: {:?}",
            permission
        ))
    }
}

#[async_trait]
impl<T> ContextEventExtractor<T> for FakePerms
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let data = ShardData::get(ctx).await?;
        let guild_id = GuildId::extract_event(ev).await?;
        data.guilds
            .get::<FakePerms, HashMap<BotPermission, Pointer<FakePermConfig>>>(guild_id)
            .await
            .map(Self)
    }
}

#[async_trait]
impl<T> Extractor<T> for FakePerms
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract(ctx: &Context, ev: &T, _: &Pointer<utils::Parser>) -> Option<Self> {
        FakePerms::extract_context_event(ctx, ev).await
    }
}
