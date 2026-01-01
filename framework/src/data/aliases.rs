use std::collections::HashMap;

use serenity::prelude::TypeMapKey;
use utils::Pointer;

#[derive(Clone)]
pub struct CommandAlias {
    pub command_name: String,
    pub args: Vec<String>,
}

impl CommandAlias {
    pub fn new<T: Into<String>>(command_name: T, args: Vec<String>) -> Self {
        Self {
            command_name: command_name.into(),
            args,
        }
    }

    pub fn with_args<T, I, S>(command_name: T, args: I) -> Self
    where
        T: Into<String>,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            command_name: command_name.into(),
            args: args.into_iter().map(|s| s.into()).collect(),
        }
    }

    pub fn empty_args<T: Into<String>>(command_name: T) -> Self {
        Self {
            command_name: command_name.into(),
            args: Vec::new(),
        }
    }

    pub fn args_as_string(&self) -> Option<String> {
        if self.args.is_empty() {
            None
        } else {
            Some(self.args.join(" "))
        }
    }
}

pub type CommandAliasMap = HashMap<String, Pointer<CommandAlias>>;

#[derive(Default, Clone)]
pub struct CommandAliases(Pointer<CommandAliasMap>);

impl TypeMapKey for CommandAliases {
    type Value = Pointer<CommandAliasMap>;
}

impl CommandAliases {
    pub async fn vec(&self) -> Vec<(String, Pointer<CommandAlias>)> {
        let map = self.0.read().await;
        map.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>()
    }

    pub async fn insert(&self, name: &str, alias: CommandAlias) {
        let mut map = self.0.write().await;
        map.insert(name.to_string(), Pointer::new(alias));
    }

    pub async fn get(&self, name: &str) -> Option<Pointer<CommandAlias>> {
        let map = self.0.read().await;
        map.get(name).cloned()
    }

    pub async fn get_cloned(&self, name: &str) -> Option<CommandAlias> {
        let map = self.0.read().await;
        match map.get(name) {
            Some(alias_ptr) => Some(alias_ptr.make_clone().await),
            None => None,
        }
    }

    pub async fn remove(&self, name: &str) {
        let mut map = self.0.write().await;
        map.remove(name);
    }

    pub async fn clear(&self) {
        let mut map = self.0.write().await;
        map.clear();
    }
}

impl From<&Pointer<CommandAliasMap>> for CommandAliases {
    fn from(ptr: &Pointer<CommandAliasMap>) -> Self {
        Self(ptr.clone())
    }
}
