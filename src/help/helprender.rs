pub struct Help {
    commands: Vec<Command>
}

impl Help {

    pub fn new() -> Help {
        Help {
            commands: Vec::new()
        }
    }

    pub fn add(mut self, command: Command) -> Help {
        self.commands.push(command);
        self
    }

    pub fn render(&self) -> String {
        let mut description = format!("**A bot jelenleg {} parancsot támogat:**", self.commands.len());

        for command in &self.commands {
            let command_description = format!("```{} {}\n{}\n{}```", 
            command.name, command.render_parameters(), command.render_example(), command.description.clone());
            description = description + command_description.as_str();
        }

        description
    }
}

pub struct Command {
    name: String,
    parameters: Option<Vec<String>>,
    example: Option<String>,
    description: String,
}

impl Command {
    pub fn new(name: &str, parameters: Option<Vec<&str>>, example: Option<&str>, description: &str) -> Command {

        Command {
            name: name.into(),
            parameters: match parameters {
                Some(parameters) => {
                    let mut vec = Vec::new();
                    for parameter in parameters {
                        vec.push(parameter.into());
                    }
                    Some(vec)
                },
                None => None,
            },
            example: match example {
                Some(example) => {
                    Some(example.into())
                },
                None => None,
            },
            description: description.into(),
        }
    }

    fn render_parameters(&self) -> String {
        match &self.parameters {
            Some(parameters) => {
                let mut parameters_string = String::new();
                for parameter in parameters {
                    parameters_string = format!("{} <{}> ", parameters_string, parameter);
                }
                parameters_string
            },
            None => String::new(),
        }
    }

    fn render_example(&self) -> String {
        match &self.example {
            Some(example) => {
                format!("{}\n", example)
            },
            None => String::new(),
        }
    }
}