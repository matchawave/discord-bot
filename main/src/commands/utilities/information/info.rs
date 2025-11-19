use framework::{
    cache::{HTTPGetter, Members},
    command::{CommandCallbackType, CommandResult, ICommand},
    extractors::InteractionOptions,
};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use serenity::all::{
    Colour, CommandOptionType, CreateCommandOption, CreateEmbed, CreateEmbedAuthor,
    FormattedTimestamp, FormattedTimestampStyle, Member, Mentionable, PartialGuild, Permissions,
    Role, RoleId, UserId,
};
use std::collections::HashMap;
use utils::{Http, MemberOption, PERMISSION_PRIORITY, command_error};

const NAME: &str = "info";
const DESCRIPTION: &str = "Get the server information";

pub fn command() -> ICommand {
    let options = CreateCommandOption::new(
        CommandOptionType::User,
        "user",
        "The user to get the avatar of",
    );
    ICommand::new(NAME, DESCRIPTION)
        .options(vec![options])
        .callbacks(vec![
            CommandCallbackType::slash(interaction),
            CommandCallbackType::legacy(legacy),
        ])
}

async fn interaction(
    http: Http,
    options: InteractionOptions,
    members: Members,
    user_id: UserId,
    guild: PartialGuild,
) -> CommandResult<CreateEmbed> {
    let Some(target) = (match options.get("user").and_then(|v| v.as_user_id()) {
        Some(id) => members.fetch(&http, (guild.id, id)).await,
        None => members.fetch(&http, (guild.id, user_id)).await,
    }) else {
        return command_error!("No member found for the given ID");
    };

    Ok(Some(execute(http, members, target, guild).await))
}

async fn legacy(
    http: Http,
    options: Vec<String>,
    members: Members,
    user_id: UserId,
    guild: PartialGuild,
) -> CommandResult<CreateEmbed> {
    let Some(target) = (match options.first().map(|id| id.parse::<MemberOption>()) {
        Some(Ok(id)) => members.fetch(&http, (guild.id, id.into())).await,
        Some(Err(e)) => return Err(e),
        None => members.fetch(&http, (guild.id, user_id)).await,
    }) else {
        return command_error!("No member found for the given ID");
    };

    Ok(Some(execute(http, members, target, guild).await))
}

async fn execute(
    http: Http,
    members: Members,
    mut target: Member,
    guild: PartialGuild,
) -> CreateEmbed {
    if target.user.accent_colour.is_none()
        && let Some(member) = members.get((guild.id, target.user.id)).await
        && let Ok(fetched) = http.get_user(target.user.id).await
    {
        member.write().await.user = fetched;
        target = member.make_clone().await;
    }

    let fields = vec![
        get_dates(&target),
        get_roles(&target.roles, &guild.roles),
        #[allow(deprecated)]
        get_permissions(guild.member_permissions(&target)),
    ];

    let avatar_url = target
        .user
        .avatar_url()
        .unwrap_or(target.user.default_avatar_url());

    let author_section =
        CreateEmbedAuthor::new(format!("{} ({})", target.user.name, target.user.id));

    let color = target.user.accent_colour.unwrap_or(Colour::BLITZ_BLUE);
    CreateEmbed::default()
        .author(author_section)
        .title(format!(
            "Information about {}{}",
            target.user.display_name(),
            { if target.user.bot { " 🤖" } else { "" } }
        ))
        .fields(fields)
        .thumbnail(avatar_url)
        .color(color)
}

fn get_permissions(perms: Permissions) -> (String, String, bool) {
    let mut perms_list = PERMISSION_PRIORITY
        .par_iter()
        .filter_map(|perm| {
            if perms.contains(*perm)
                && let Some(name) = perm.get_permission_names().into_iter().next()
            {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    perms_list.truncate(3);
    let title = if perms_list.len() < 2 {
        "Permission".to_string()
    } else {
        format!("Permissions ({})", perms_list.len())
    };
    let output = if perms_list.is_empty() {
        "None".to_string()
    } else {
        perms_list.join(", ")
    };
    (title, output, false)
}

fn get_dates(member: &Member) -> (String, String, bool) {
    let joined_at_string = match member.joined_at {
        Some(d) => format!(
            "{} ({})",
            d.format("%Y-%m-%d %H:%M:%S"),
            FormattedTimestamp::new(d, Some(FormattedTimestampStyle::RelativeTime))
        ),
        None => "Unknown".to_string(),
    };
    let created_at_string = {
        let created_at = member.user.created_at();
        format!(
            "{} ({})",
            created_at.format("%Y-%m-%d %H:%M:%S"),
            FormattedTimestamp::new(created_at, Some(FormattedTimestampStyle::RelativeTime))
        )
    };
    (
        "Dates".to_string(),
        format!(
            "Joined: {}\nCreated: {}",
            joined_at_string, created_at_string
        ),
        false,
    )
}

fn get_roles(user_roles: &[RoleId], guild_roles: &HashMap<RoleId, Role>) -> (String, String, bool) {
    let mut roles = user_roles
        .par_iter()
        .filter_map(|role_id| guild_roles.get(role_id))
        .collect::<Vec<_>>();

    roles.par_sort_by(|a, b| b.position.cmp(&a.position));

    let title = if roles.len() < 2 {
        "Role".to_string()
    } else {
        format!("Roles ({})", roles.len())
    };

    let output = if roles.len() > 7 {
        roles.truncate(7);
        format!("{} ...", write_roles(&roles))
    } else {
        write_roles(&roles)
    };

    (title, output, false)
}

fn write_roles(roles: &[&Role]) -> String {
    if roles.is_empty() {
        "None".to_string()
    } else {
        roles
            .par_iter()
            .map(|r| r.mention().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
