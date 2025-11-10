use framework::command::CommandManager;
mod example;
mod utilities;

pub fn create_command_handler() -> CommandManager {
    let command_manager = CommandManager::default();
    command_manager.insert_all(utilities::module());
    command_manager
}
