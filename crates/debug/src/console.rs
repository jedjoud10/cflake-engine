// A console that will let the user write debug commands
#[derive(Default)]
pub struct Console {
    // Some template commands that we check against
    pub template_command: Vec<Command>,
    pub active: bool,

    // The currently sent command (Note: We can only have one sent command at a time)
    sent_command: Option<Command>,
}

// Listen to commands and send a message if we received one
impl Console {
    // Detect the currently written command and send the event to all command listeneners
    pub fn detect_command(&mut self, text: String) -> Option<()> {
        // Clear the sent command
        self.sent_command = None;
        // Get the current command's name
        let name = text.split(" ").nth(0)?;
        // Get the inputs
        let inputs = text.split(" ").collect::<Vec<&str>>()[1..].to_vec();

        // The final command
        let mut final_command: Option<Command> = None;
        for command in self.template_command.iter() {
            if name == &command.name {
                // We found a matching command!
                let mut final_assossiated_inputs: Vec<CommandInput> = Vec::new();
                for associated_input in command.inputs.iter() {
                    // Get the name and the actual value
                    let index = inputs.iter().position(|x| x.to_string() == associated_input.short_name)?;
                    
                    // Get the value
                    let value: CommandInputEnum = {
                        let value_string = inputs.get(index + 1)?;
                        // Parse the string
                        match associated_input.input {
                            CommandInputEnum::F32(_) => {
                                // Parse to f32
                                let output = value_string.parse::<f32>().ok()?;
                                CommandInputEnum::F32(output)                                
                            },
                            CommandInputEnum::I32(_) => {
                                // Parse to i32
                                let output = value_string.parse::<i32>().ok()?;
                                CommandInputEnum::I32(output)
                            },
                            CommandInputEnum::BOOL(_) => {
                                // Parse to bool
                                let output = value_string.parse::<bool>().ok()?;
                                CommandInputEnum::BOOL(output)
                            },
                        }
                    };
                    // Finally push
                    final_assossiated_inputs.push(CommandInput { short_name: associated_input.short_name.clone(), input: value });
                }
                final_command = Some(Command { name: command.name.clone(), inputs: final_assossiated_inputs });
                break;                
            }
        }
        // If the final command is not none then send the message
        match final_command {
            Some(x) => { self.send_command(x); }
            _ => { }
        }
        Some(())
    }
    // Send a message to all the console command receivers
    pub fn send_command(&mut self, command: Command) {
        self.sent_command = Some(command);
    }
    // Register for a specific command
    pub fn register_template_command(&mut self, command: Command) {
        self.template_command.push(command);
    }
    // Check for a specific received command
    pub fn listen_command(&mut self, command_name: &str) -> Option<Command> {
        match self.sent_command.clone() {
            Some(a) => {
                // Check if this is the right command
                if a.name == command_name.to_string() {
                    let output = Some(a);
                    self.sent_command = None;
                    output
                } else { None }
            },
            None => None,
        }
    }
}

// A simple command
#[derive(Default, Clone)]

pub struct Command {
    // The command's name
    pub name: String,
    // Associated inputs with this command
    pub inputs: Vec<CommandInput>,
}

impl Command {
    // Get a specific input from the command
    pub fn get_input(&self, input_short_name: &str) -> Option<&CommandInputEnum> {
        match self.inputs.iter().find(|x| x.short_name == input_short_name) {
            Some(x) => Some(&x.input),
            None => None,
        }
    }
}

// An associated input with each command
#[derive(Clone)]
pub struct CommandInput {
    pub short_name: String,
    pub input: CommandInputEnum,
}

impl CommandInput {
    // Create a new command input
    pub fn new<T: CommandInputTrait>(name: &str) -> Self {
        return CommandInput { short_name: name.to_string(), input: T::get_default_input() }
    }
}

// A command input enum
#[derive(Clone, Debug)]
pub enum CommandInputEnum {
    F32(f32),
    I32(i32),
    BOOL(bool),
}


// A command input trait
pub trait CommandInputTrait {
    fn get_default_input() -> CommandInputEnum;
}

impl CommandInputTrait for f32 {
    fn get_default_input() -> CommandInputEnum {
        CommandInputEnum::F32(0.0)
    }
}

impl CommandInputTrait for i32 {
    fn get_default_input() -> CommandInputEnum {
        CommandInputEnum::I32(0)
    }
}

impl CommandInputTrait for bool {
    fn get_default_input() -> CommandInputEnum {
        CommandInputEnum::BOOL(false)
    }
}