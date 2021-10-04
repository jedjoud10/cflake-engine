// A console that will let the user write debug commands
#[derive(Default)]
pub struct Console {
    // All the commands that we need to listen to
    pub commands: Vec<Command>,
    pub current_written_buffer: String,
    pub active: bool,

    // The currently sent command (Note: We can only have one sent command at a time)
    sent_command: Option<Command>,
}

// Listen to commands and send a message if we received one
impl Console {
    // Update the console's current buffer
    pub fn update_buffer(&mut self, new_data: &str) {
        if self.active {
            // Append
            self.current_written_buffer += new_data;
        }
    }
    // We pressed the enter key, check for every command
    pub fn finalize_buffer(&mut self) {
        // Clear the sent command
        self.sent_command = None;
        // Get the current command's name
        let name = self.current_written_buffer.split(" ").nth(0).unwrap();
        // Get the inputs
        let inputs = self.current_written_buffer.split(" ").collect::<Vec<&str>>()[1..].to_vec();

        // The final command
        let mut final_command: Option<Command> = None;
        for command in self.commands.iter() {
            if name == &command.name {
                // We found a matching command!
                let mut final_assossiated_inputs: Vec<CommandInput> = Vec::new();
                let mut i = 0;
                for associated_input in command.inputs.iter() {
                    // Get the name and the actual value
                    let index = inputs.iter().position(|x| x.to_string() == associated_input.short_name).unwrap();
                    
                    // Get the value
                    let value: CommandInputEnum = {
                        let value_string = inputs[index + 1];
                        // Parse the string
                        match associated_input.input {
                            CommandInputEnum::F32(_) => {
                                // Parse to f32
                                let output = value_string.parse::<f32>().unwrap();
                                CommandInputEnum::F32(output)                                
                            },
                            CommandInputEnum::I32(_) => {
                                // Parse to i32
                                let output = value_string.parse::<i32>().unwrap();
                                CommandInputEnum::I32(output)
                            },
                            CommandInputEnum::BOOL(_) => {
                                // Parse to bool
                                let output = value_string.parse::<bool>().unwrap();
                                CommandInputEnum::BOOL(output)
                            },
                        }
                    };
                    // Finally push
                    final_assossiated_inputs.push(CommandInput { short_name: associated_input.short_name.clone(), input: value });
                    i += 1;
                }
                final_command = Some(Command { name: command.name.clone(), inputs: final_assossiated_inputs });
                break;                
            }
        }
        // If the final command is not none then send the message
        match final_command {
            Some(x) => { self.send_command(x); }
            _ => {}
        }
    }
    // Send a message to all the console command receivers
    pub fn send_command(&mut self, command: Command) {
        self.sent_command = Some(command);
    }
    // Check for a specific received command
    pub fn receive_command(&self, command_name: &str) -> Option<&Command> {
        match self.sent_command.as_ref() {
            Some(a) => {
                // Check if this is the right command
                if a.name == command_name.to_string() {
                    Some(a)
                } else { None }
            },
            None => None,
        }
    }
}

// A simple command
#[derive(Default)]

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
pub struct CommandInput {
    pub short_name: String,
    pub input: CommandInputEnum,
}

// A command input enum
pub enum CommandInputEnum {
    F32(f32),
    I32(i32),
    BOOL(bool),
}