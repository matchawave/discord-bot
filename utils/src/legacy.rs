use chrono::TimeDelta;
use serenity::all::{ChannelId, RoleId, UserId};

use crate::{ResponseError, command_error};

macro_rules! legacy_option {
    ($($name:ident, $inner:ty, $callback:expr;)*) => {
        $(
            pub struct $name($inner);

            impl From<$name> for $inner {
                fn from(value: $name) -> Self {
                    value.0
                }
            }

            impl std::ops::Deref for $name {
                type Target = $inner;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl std::str::FromStr for $name {
                type Err = ResponseError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    $callback(s)
                }
            }
        )*
    };
}

legacy_option! {
    TimeOption, TimeDelta, |s: &str| {
        let mut current_number = String::new();
        let mut current_unit = String::new();
        let mut is_parsing_number = true;

        for (i, c) in s.chars().enumerate() {
            if i == 0 && !c.is_ascii_digit() {
                return command_error!("Invalid duration format: `{}`", s);
            }
            if c.is_ascii_digit() && is_parsing_number {
                current_number.push(c);
            } else if c.is_ascii_alphabetic() {
                current_unit.push(c);
                is_parsing_number = false;
            } else {
                return command_error!("Invalid duration format: `{}`", s);
            }
        }

        if current_number.is_empty() && current_unit.is_empty() {
            return command_error!("Invalid duration format: `{}`", s);
        }
        let number: i64 = current_number.parse().map_err(|_| {
            ResponseError::new(format!("Invalid number in duration: `{}`", current_number))
        })?;
        current_number.clear();

        let seconds = match current_unit.as_str() {
            "s" | "sec" | "secs" | "second" | "seconds" => number,
            "m" | "min" | "mins" | "minute" | "minutes" => number * 60,
            "h" | "hr" | "hrs" | "hour" | "hours" => number * 3600,
            "d" | "day" | "days" => number * 86400,
            "w" | "week" | "weeks" => number * 604800,
            _ => {
                return command_error!("Invalid time unit in duration: `{}`", current_unit);
            }
        };

        Ok(Self(TimeDelta::seconds(seconds)))
    };
    IntegerOption, i64, |s: &str| {
        let number: i64 = s
            .parse()
            .map_err(|_| ResponseError::new(format!("Invalid integer value: `{}`", s)))?;
        Ok(Self(number))
    };
    BooleanOption, bool, |s: &str| {
        let value = s.to_lowercase();
        let boolean = match value.as_str() {
            "true" | "yes" | "1" => true,
            "false" | "no" | "0" => false,
            _ => {
                return command_error!("Invalid boolean value: `{}`", s);
            }
        };
        Ok(Self(boolean))
    };
    ChannelOption, ChannelId, |s: &str| {
        if s.is_empty() {
            return command_error!("MemberOption cannot be created from empty string");
        }
        if s.starts_with('<') && s.ends_with('>') {
            if s.starts_with("<@&") {
                return command_error!(
                    "Expected a channel mention, but got a role mention: `{}`",
                    s
                );
            }
            if s.starts_with("<@") {
                return command_error!(
                    "Invalid role channel format, but got a user mention: `{}`",
                    s
                );
            }
        }
        let s = s.trim_start_matches("<#").trim_end_matches('>');
        let channel_id: u64 = s
            .parse()
            .map_err(|_| ResponseError::new(format!("Invalid channel ID value: `{}`", s)))?;
        Ok(Self(ChannelId::from(channel_id)))
    };
    RoleOption, RoleId, |s: &str| {
        if s.is_empty() {
            return command_error!("MemberOption cannot be created from empty string");
        }
        if s.starts_with('<') && s.ends_with('>') {
            if s.starts_with("<#") {
                return command_error!(
                    "Expected a role mention, but got a channel mention: `{}`",
                    s
                );
            }
            if s.starts_with("<@") {
                return command_error!("Expected a role mention, but got a user mention: `{}`", s);
            }
        }
        let s = s.trim_start_matches("<@&").trim_end_matches('>');
        let role_id: u64 = s
            .parse()
            .map_err(|_| ResponseError::new(format!("Invalid role ID value: `{}`", s)))?;
        Ok(Self(RoleId::from(role_id)))
    };
    MemberOption, UserId, |s: &str| {
        if s.is_empty() {
            return command_error!("MemberOption cannot be created from empty string");
        }
        if s.starts_with('<') && s.ends_with('>') {
            if s.starts_with("<#") {
                return command_error!(
                    "Expected a member mention, but got a channel mention: `{}`",
                    s
                );
            }
            if s.starts_with("<@&") {
                return command_error!(
                    "Expected a member mention, but got a role mention: `{}`",
                    s
                );
            }
        }
        let s = s.trim_start_matches("<@").trim_end_matches('>');
        let user_id: u64 = s
            .parse()
            .map_err(|_| ResponseError::new(format!("Invalid user ID value: `{}`", s)))?;
        Ok(Self(UserId::from(user_id)))
    };
}
