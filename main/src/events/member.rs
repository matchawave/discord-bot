use serenity::{
    all::{GuildId, Member},
    futures::StreamExt,
};
use utils::{Http, error, info};

use crate::data::member::MembersInfo;

pub async fn get_members(guild_id: GuildId, members_list: MembersInfo, http: Http) {
    tokio::spawn(async move {
        let mut members = guild_id.members_iter(http).boxed();
        info!("Fetching members for guild {}...", guild_id);
        let mut total = 0;
        while let Some(member_res) = members.next().await {
            match member_res {
                Ok(member) => {
                    members_list.add(&member).await;
                    total += 1;
                }
                Err(err) => {
                    error!("Failed to fetch member in guild {}: {}", guild_id, err);
                }
            }
        }
        info!("Fetched {} members for guild {}", total, guild_id);
    });
}

pub async fn remove_members(guild_id: GuildId, members_list: MembersInfo) {
    members_list.remove(&guild_id).await;
}

pub async fn add_member(member: Member, members_list: MembersInfo) {
    members_list.add(&member).await;
}

pub async fn subtract_member(member: Member, members_list: MembersInfo) {
    members_list.subtract(&member).await;
}
