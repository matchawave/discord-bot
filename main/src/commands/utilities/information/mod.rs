use framework::command::ICommand;

mod avatar;
mod banner;
mod info;
mod ping;
mod serverinfo;
pub fn register() -> Vec<ICommand> {
    vec![
        ping::command(),
        avatar::command(),
        serverinfo::command(),
        info::command(),
        banner::command(),
    ]
}
