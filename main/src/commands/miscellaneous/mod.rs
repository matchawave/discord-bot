use framework::command::ICommand;

mod afk;

pub fn module() -> Vec<ICommand> {
    vec![afk::command()]
}
