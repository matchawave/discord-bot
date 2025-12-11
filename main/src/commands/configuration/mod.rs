use framework::command::ICommand;

mod prefix;

pub fn module() -> Vec<ICommand> {
    vec![prefix::command()]
}
