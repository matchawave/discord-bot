use crate::command::CommandManager;
use serenity::prelude::TypeMapKey;

#[macro_export]
macro_rules! sharded_data {
    ($struct_name: ident, $key_type:ident, { $($setter:tt)* }) => {
        pub struct $struct_name(pub std::sync::Arc<serenity::prelude::TypeMap>);
        impl $struct_name {
            pub fn set(shards: usize, data: &mut serenity::prelude::TypeMap) {
                let mut data_vec = Vec::with_capacity(shards);
                for _shard in 0..shards {
                    let mut type_map = serenity::prelude::TypeMap::new();
                    $($setter)*(&mut type_map);
                    data_vec.push(std::sync::Arc::new(type_map));
                }
                data.insert::<$key_type>(std::sync::Arc::new(data_vec));
            }

            pub fn initialize<F>(shards: usize, data: &mut serenity::prelude::TypeMap, callback: F)
            where
                F: FnOnce(&mut serenity::prelude::TypeMap) + Copy,
            {
                let mut data_vec = Vec::with_capacity(shards);
                for _shard in 0..shards {
                    let mut type_map = serenity::prelude::TypeMap::new();
                    callback(&mut type_map);
                    $($setter)*(&mut type_map);
                    data_vec.push(std::sync::Arc::new(type_map));
                }
                data.insert::<$key_type>(std::sync::Arc::new(data_vec));
            }

            pub async fn get(
                data: &utils::DataType,
                shard_id: serenity::all::ShardId,
            ) -> Option<std::sync::Arc<serenity::prelude::TypeMap>> {
                let data = data.read().await;
                let datas = data.get::<$key_type>().cloned()?;
                datas.get(shard_id.get() as usize).map(|p| p.clone())
            }
        }

        #[serenity::async_trait]
        impl $crate::extractors::Extractor<serenity::all::Event> for $struct_name {
            async fn extract(ctx: &serenity::all::Context, _e: &serenity::all::Event, _p: &utils::Pointer<utils::Parser>) -> Option<Self> {
                let shard_id = ctx.shard_id;
                let pointer = $struct_name::get(&ctx.data, shard_id).await;
                pointer.map($struct_name)
            }
        }

        #[serenity::async_trait]
        impl $crate::extractors::Extractor<$crate::command::CommandAction> for $struct_name {
            async fn extract(ctx: &serenity::all::Context, _a: &$crate::command::CommandAction, _p: &utils::Pointer<utils::Parser>) -> Option<Self> {
                let shard_id = ctx.shard_id;
                let pointer = $struct_name::get(&ctx.data, shard_id).await;
                pointer.map($struct_name)
            }
        }
    };
}

pub struct Commands;
impl TypeMapKey for Commands {
    type Value = CommandManager;
}
