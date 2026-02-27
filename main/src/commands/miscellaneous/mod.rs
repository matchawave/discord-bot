use framework::command::ICommand;

mod afk;
mod birthday;

pub fn module() -> Vec<ICommand> {
    vec![afk::command(), birthday::command()]
}
