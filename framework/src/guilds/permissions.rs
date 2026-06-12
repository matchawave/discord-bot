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
            if let Some(perm_config_ptr) = read.get(permission) {
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
            let mut has_role = false;
            if let Some(perm_config_ptr) = read.get(permission) {
                let perm_config = perm_config_ptr.read().await;
                if perm_config.users.par_iter().any(|&id| id == member.user.id) {
                    continue;
                }
                for role in &perm_config.roles {
                    if member.roles.par_iter().any(|r| r == role) {
                        has_role = true;
                        break;
                    }
                }
            }
            if !has_role {
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

    pub async fn add_user_to_permissions(
        &self,
        permissions: &[BotPermission],
        user_id: UserId,
    ) -> Result<(), String> {
        let read = self.0.read().await;
        let mut list_of_already_perms = Vec::new();
        for &permission in permissions {
            if let Some(perm_config_ptr) = read.get(&permission) {
                let mut perm_config = perm_config_ptr.write().await;
                if !perm_config.users.par_iter().any(|&id| id == user_id) {
                    perm_config.users.push(user_id);
                } else {
                    // Keep track of permissions that the user already has to return an error if all permissions were already present
                    list_of_already_perms.push(permission);
                }
            } else {
                return Err(format!(
                    "Permission config not found for permission: {:?}",
                    permission
                ));
            }
        }
        if list_of_already_perms.len() == permissions.len() {
            return Err("User already has all of these permissions".to_string());
        } else if !list_of_already_perms.is_empty() {
            return Err(format!(
                "User already has the following permissions: {:?}",
                list_of_already_perms
            ));
        }
        Ok(())
    }

    pub async fn add_role_to_permission(
        &self,
        permission: &[BotPermission],
        role_id: RoleId,
    ) -> Result<(), String> {
        let read = self.0.read().await;
        let mut list_of_already_perms = Vec::new();
        for &perm in permission {
            if let Some(perm_config_ptr) = read.get(&perm) {
                let mut perm_config = perm_config_ptr.write().await;
                if !perm_config.roles.par_iter().any(|&id| id == role_id) {
                    perm_config.roles.push(role_id);
                } else {
                    // Keep track of permissions that the role already has to return an error if all permissions were already present
                    list_of_already_perms.push(perm);
                }
            } else {
                return Err(format!(
                    "Permission config not found for permission: {:?}",
                    perm
                ));
            }
        }
        if list_of_already_perms.len() == permission.len() {
            return Err("Role already has all of these permissions".to_string());
        } else if !list_of_already_perms.is_empty() {
            return Err(format!(
                "Role already has the following permissions: {:?}",
                list_of_already_perms
            ));
        }
        Ok(())
    }

    pub async fn remove_user_from_permission(
        &self,
        permission: &[BotPermission],
        user_id: UserId,
    ) -> Result<(), String> {
        let read = self.0.read().await;
        let mut list_of_missing_perms = Vec::new();
        for &perm in permission {
            if let Some(perm_config_ptr) = read.get(&perm) {
                let mut perm_config = perm_config_ptr.write().await;
                if let Some(pos) = perm_config.users.iter().position(|&id| id == user_id) {
                    perm_config.users.remove(pos);
                } else {
                    list_of_missing_perms.push(perm);
                }
            } else {
                return Err(format!(
                    "Permission config not found for permission: {:?}",
                    perm
                ));
            }
        }
        if !list_of_missing_perms.is_empty() {
            return Err(format!(
                "User does not have the following permissions: {:?}",
                list_of_missing_perms
            ));
        }
        Ok(())
    }

    pub async fn remove_role_from_permission(
        &self,
        permission: &[BotPermission],
        role_id: RoleId,
    ) -> Result<(), String> {
        let read = self.0.read().await;
        let mut list_of_missing_perms = Vec::new();
        for &perm in permission {
            if let Some(perm_config_ptr) = read.get(&perm) {
                let mut perm_config = perm_config_ptr.write().await;
                if let Some(pos) = perm_config.roles.iter().position(|&id| id == role_id) {
                    perm_config.roles.remove(pos);
                } else {
                    list_of_missing_perms.push(perm);
                }
            } else {
                return Err(format!(
                    "Permission config not found for permission: {:?}",
                    perm
                ));
            }
        }
        if list_of_missing_perms.len() == permission.len() {
            return Err("Role does not have any of these permissions".to_string());
        } else if !list_of_missing_perms.is_empty() {
            return Err(format!(
                "Role does not have the following permissions: {:?}",
                list_of_missing_perms
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl<T> ContextEventExtractor<T> for FakePerms
where
    T: Send + Sync + 'static,
    GuildId: EventExtractor<T>,
{
    async fn extract_context_event(ctx: &Context, ev: &T) -> Option<Self> {
        let data = ShardData::get(ctx.shard_id, &ctx.data).await?;
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
