use framework::command::ICommand;

mod afk;

pub fn module() -> Vec<ICommand> {
    let mut commands = Vec::new();
    commands.push(afk::command());
    commands
}
