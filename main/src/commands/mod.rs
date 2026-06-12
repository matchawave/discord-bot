use framework::{
    command::{CommandManager, CommandManagerBuilder},
    guilds::{HTTPGetter, Members},
};
use serenity::all::{Colour, CreateEmbed, CreateEmbedAuthor, Member, User, UserId};
use utils::HttpType;

use crate::permissions;
mod configuration;
mod example;
mod miscellaneous;
mod utilities;

pub fn create_command_handler() -> CommandManager {
    CommandManagerBuilder::default()
        .add_commands(utilities::module())
        .add_commands(configuration::module())
        .add_commands(miscellaneous::module())
        .set_permission_callback(permissions::callback)
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

#[macro_export]
macro_rules! success {
    ($user_id:expr, $($arg:tt)*) => {
        {
            use serenity::all::Mentionable;
            let user_id = $user_id;
            serenity::all::CreateEmbed::default()
                .colour((39u8, 245u8, 132u8))
                .description(format!("{}: {}", user_id.mention().to_string(), format!($($arg)*)))
        }
    };
    ($($arg:tt)*) => {
        serenity::all::CreateEmbed::default()
            .colour((39u8, 245u8, 132u8))
            .description(format!($($arg)*))
    };
}
