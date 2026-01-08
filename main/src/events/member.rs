use serenity::{
    all::{GuildId, Member},
    futures::StreamExt,
};
use utils::{HttpType, error, info};

use crate::{data::member_list::MemberList, global::shard_list::ShardData};

pub async fn get_members(
    guild_id: GuildId,
    members_list: MemberList,
    http: HttpType,
    shard_data: ShardData,
) {
    shard_data.add_server(guild_id).await;
    tokio::spawn(async move {
        let mut members = guild_id.members_iter(http).boxed();
        info!("Fetching members for guild {}...", guild_id);
        let mut total = 0;
        while let Some(member_res) = members.next().await {
            match member_res {
                Ok(member) => {
                    members_list.insert(&member).await;
                    shard_data.add_member(1).await;
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

pub async fn add_member(member: Member, members_list: MemberList, shard_data: ShardData) {
    members_list.insert(&member).await;
    shard_data.add_member(1).await;
}

pub async fn subtract_member(member: Member, members_list: MemberList, shard_data: ShardData) {
    members_list.remove(member.user.id).await;
    shard_data.remove_member(1).await;
}
