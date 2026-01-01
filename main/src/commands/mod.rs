use framework::{
    command::{CommandManager, CommandManagerBuilder},
    guilds::{HTTPGetter, Members},
};
use serenity::all::{CreateEmbedAuthor, Member, User, UserId};
use utils::HttpType;
mod configuration;
mod example;
mod utilities;

pub fn create_command_handler() -> CommandManager {
    CommandManagerBuilder::default()
        .add_commands(utilities::module())
        .add_commands(configuration::module())
        .build()
}

pub fn author_embed(user: &User) -> CreateEmbedAuthor {
    CreateEmbedAuthor::new(user.name.clone())
        .icon_url(user.avatar_url().unwrap_or(user.default_avatar_url()))
}

pub async fn get_author_embed(
    http: HttpType,
    members: Members,
    target: &Member,
    author_id: UserId,
) -> Option<CreateEmbedAuthor> {
    if author_id == target.user.id {
        Some(author_embed(&target.user))
    } else {
        members
            .fetch(&http, (target.guild_id, author_id))
            .await
            .map(|author| author_embed(&author.user))
    }
}
