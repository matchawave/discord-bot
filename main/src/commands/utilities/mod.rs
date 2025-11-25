use framework::command::ICommand;

mod information;
mod snipes;

pub fn module() -> Vec<ICommand> {
    let mut commands = Vec::new();
    commands.extend(information::register());
    commands.extend(snipes::register());
    commands
}
