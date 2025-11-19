use framework::{
    cache::{HTTPGetter, Members},
    command::CommandManager,
};
use serenity::all::{CreateEmbedAuthor, Member, UserId};
use utils::Http;
mod configuration;
mod example;
mod utilities;

pub fn create_command_handler() -> CommandManager {
    let command_manager = CommandManager::default();
    command_manager.insert_all(utilities::module());
    command_manager.insert_all(configuration::module());
    command_manager
}

pub fn author_embed(member: &Member) -> CreateEmbedAuthor {
    CreateEmbedAuthor::new(member.user.name.clone()).icon_url(
        member
            .user
            .avatar_url()
            .unwrap_or(member.user.default_avatar_url()),
    )
}

pub async fn get_author_embed(
    http: Http,
    members: Members,
    target: &Member,
    author_id: UserId,
) -> Option<CreateEmbedAuthor> {
    if author_id == target.user.id {
        Some(author_embed(target))
    } else {
        members
            .fetch(&http, (target.guild_id, author_id))
            .await
            .map(|author| author_embed(&author))
    }
}
