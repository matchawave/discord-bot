use framework::command::ICommand;

mod information;

pub fn module() -> Vec<ICommand> {
    let mut commands = Vec::new();
    commands.extend(information::register());
    commands
}
