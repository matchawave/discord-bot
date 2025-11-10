use framework::command::ICommand;

mod avatar;
mod banner;
mod ping;
mod serverinfo;
pub fn register() -> Vec<ICommand> {
    vec![
        ping::command(),
        avatar::command(),
        serverinfo::command(),
        banner::command(),
    ]
}
