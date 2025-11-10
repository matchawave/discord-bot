use std::str::FromStr;

use chrono::TimeDelta;
use serenity::all::{ChannelId, GuildChannel, Member, Role, RoleId, UserId};

use crate::{ResponseError, command_error};

pub struct TimeOption(TimeDelta);
impl From<TimeOption> for TimeDelta {
    fn from(value: TimeOption) -> Self {
        value.0
    }
}
impl FromStr for TimeOption {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
    }
}

pub struct IntegerOption(i64);
impl From<IntegerOption> for i64 {
    fn from(value: IntegerOption) -> Self {
        value.0
    }
}
impl FromStr for IntegerOption {
    type Err = ResponseError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let number: i64 = value
            .parse()
            .map_err(|_| ResponseError::new(format!("Invalid integer value: `{}`", value)))?;
        Ok(Self(number))
    }
}

pub struct BooleanOption(bool);
impl From<BooleanOption> for bool {
    fn from(value: BooleanOption) -> Self {
        value.0
    }
}
impl FromStr for BooleanOption {
    type Err = ResponseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.to_lowercase();
        let boolean = match value.as_str() {
            "true" | "yes" | "1" => true,
            "false" | "no" | "0" => false,
            _ => {
                return Err(ResponseError::new(format!(
                    "Invalid boolean value: `{}`",
                    value
                )));
            }
        };
        Ok(Self(boolean))
    }
}

pub struct ChannelOption(ChannelId); // Needs Channels
impl From<ChannelOption> for ChannelId {
    fn from(value: ChannelOption) -> Self {
        value.0
    }
}
impl FromStr for ChannelOption {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
    }
}

pub struct RoleOption(RoleId); // Needs Guild
impl From<RoleOption> for RoleId {
    fn from(value: RoleOption) -> Self {
        value.0
    }
}
impl FromStr for RoleOption {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
    }
}
pub struct MemberOption(UserId); // Needs Members
impl From<MemberOption> for UserId {
    fn from(value: MemberOption) -> Self {
        value.0
    }
}
impl FromStr for MemberOption {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
    }
}
