use framework::command::ICommand;

mod prefix;

pub fn module() -> Vec<ICommand> {
    let mut commands = vec![prefix::command()];
    // commands.extend(information::register());
    commands
}
