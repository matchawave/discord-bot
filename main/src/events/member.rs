use serenity::{
    all::{GuildId, Member, UserId},
    futures::StreamExt,
};
use utils::{HttpType, error, info};

use crate::{data::member_list::MemberList, global::backend_http::BackendHttp};

pub async fn get_members(
    guild_id: GuildId,
    members_list: MemberList,
    http: HttpType,
    backend_http: BackendHttp,
) {
    tokio::spawn(async move {
        let mut member_list: Vec<String> = vec![];
        let mut members = guild_id.members_iter(http).boxed();

        info!("Fetching members for guild {}...", guild_id);
        let mut total = 0;
        while let Some(member_res) = members.next().await {
            match member_res {
                Ok(member) => {
                    members_list.insert(&member).await;
                    member_list.push(member.user.id.to_string());
                    total += 1;
                }
                Err(err) => {
                    error!("Failed to fetch member in guild {}: {}", guild_id, err);
                }
            }

            if member_list.len() >= 100 {
                store_member(&backend_http, guild_id, &member_list).await;
                member_list.clear();
            }
        }
        if !member_list.is_empty() {
            store_member(&backend_http, guild_id, &member_list).await;
        }
        info!("Fetched {} members for guild {}", total, guild_id);
    });
}

async fn store_member(backend_http: &BackendHttp, guild_id: GuildId, member_list: &Vec<String>) {
    let path = format!("api/guild/{}/member", guild_id);
    if let Err(err) = backend_http.post::<_, ()>(&path, member_list).await {
        error!("Failed to send member list for guild {}: {}", guild_id, err);
    }
    info!(
        "Stored {} members for guild {}",
        member_list.len(),
        guild_id
    );
}

pub async fn add_member(
    guild_id: GuildId,
    member: Member,
    members_list: MemberList,
    backend_http: BackendHttp,
) {
    members_list.insert(&member).await;
    let path = format!("api/guild/{}/member/{}", guild_id, member.user.id);
    if let Err(err) = backend_http.post::<(), ()>(&path, &()).await {
        error!("Failed to send member list for guild {}: {}", guild_id, err);
    }
}

pub async fn subtract_member(
    guild_id: GuildId,
    member: Member,
    members_list: MemberList,
    backend_http: BackendHttp,
) {
    members_list.remove(member.user.id).await;
    let path = format!("api/guild/{}/member/{}", guild_id, member.user.id);
    if let Err(err) = backend_http.delete::<(), ()>(&path, &()).await {
        error!("Failed to send member list for guild {}: {}", guild_id, err);
    }
}
