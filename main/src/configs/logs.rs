use utils::DataType;

macro_rules! create_log_struct {
    ($($name:ident, { $($field:ident),* $(,)? };)*) => {
        $(
            #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
            pub struct $name {
                $(pub $field: (bool, Option<serenity::all::ChannelId>)),*
            }

            impl serenity::prelude::TypeMapKey for $name {
                type Value = utils::Pointer<$name>;
            }
        )*
    };
}

create_log_struct!(
    MessageLog, { edit, delete, command, bulk_delete, };
    VoiceLog, { join, leave, switch, };
    ModerationLog, { ban, unban, kick, mute, unmute, warn, timout, untimeout, };
    MemberLog, { join, leave, update, };
    ChannelLog, { create, delete, update,};
    RoleLog, { create, delete, update,};
    EmojiLog, { create, delete, update, };
    GuildLog, { update, invites };
);

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LogConfig {
    pub message: Option<MessageLog>,
    pub voice: Option<VoiceLog>,
    pub moderation: Option<ModerationLog>,
    pub member: Option<MemberLog>,
    pub channel: Option<ChannelLog>,
    pub role: Option<RoleLog>,
    pub emoji: Option<EmojiLog>,
    pub guild: Option<GuildLog>,
}

macro_rules! populate_config {
    ($log_config:expr, $data_read:expr, [$(($log_type:ty, $field_name:ident)),* $(,)?]) => {
        $(
            if let Some(log_value) = $data_read.get::<$log_type>() {
                $log_config.$field_name = Some(log_value.make_clone().await);
            }
        )*
    };
}

impl LogConfig {
    async fn from_data(data: &DataType) -> Self {
        let data_read = data.read().await;
        let mut log_config = LogConfig::default();

        populate_config!(
            log_config,
            data_read,
            [
                (MessageLog, message),
                (VoiceLog, voice),
                (ModerationLog, moderation),
                (MemberLog, member),
                (ChannelLog, channel),
                (RoleLog, role),
                (EmojiLog, emoji),
                (GuildLog, guild),
            ]
        );

        log_config
    }
}
