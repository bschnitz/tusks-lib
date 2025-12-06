pub struct TusksCLI {
    pub tasks: Tasks,
    pub separator: &'static str, // konfigurierbarer Trenner
}

impl TusksCLI {
    pub fn new(tasks: Tasks) -> Self {
        Self { tasks, separator: "." }
    }

    pub fn with_separator(mut self, sep: &'static str) -> Self {
        self.separator = sep;
        self
    }

    pub fn start(&mut self, argv: Vec<String>) {
        let app = self.build_cli_app_with_subcommands();
        
        match app.try_get_matches_from(argv) {
            Ok(matches) => {
                self.handle_subcommand_matches(&matches);
            }
            Err(err) => {
                err.print().unwrap();
            }
        }
    }

    pub fn start_with_env(&mut self) {
        let app = self.build_cli_app_with_subcommands();
        
        match app.try_get_matches() {
            Ok(matches) => {
                self.handle_subcommand_matches(&matches);
            }
            Err(err) => {
                err.print().unwrap();
            }
        }
    }

    fn build_cli_app_with_subcommands(&self) -> Command {
        let mut app = Command::new("TasksCLI")
            .version("1.0")
            .author("Your Name")
            .about("Task execution CLI with subcommands");

        for (task_path, task) in self.tasks.iter() {
            let subcommand_name = task_path.join(self.separator);
            let mut subcommand = Command::new(subcommand_name.clone())
                .about(format!("Execute task: {}", task.name));

            // Argumente hinzufügen wie gehabt ...
            for (arg_name, argument) in &task.arguments {
                let help_text = format!("Type: {} - {}",
                    argument.arg_type,
                    argument.default_value.as_ref()
                        .map(|d| format!("Default: {}", d))
                        .unwrap_or_else(|| "Required".to_string()));

                let mut arg = Arg::new(arg_name.clone())
                    .long(arg_name.clone())
                    .help(help_text)
                    .value_parser(value_parser!(String));

                if let Some(default) = &argument.default_value {
                    arg = arg.default_value(default.clone());
                } else {
                    arg = arg.required(false);
                }

                subcommand = subcommand.arg(arg);
            }

            app = app.subcommand(subcommand);
        }

        app
    }

    fn find_task_by_subcommand(&self, subcommand_name: &str) -> Option<(Vec<String>, &Task)> {
        for (task_path, task) in self.tasks.iter() {
            let expected_subcommand = task_path.join(self.separator);
            if expected_subcommand == subcommand_name {
                return Some((task_path.clone(), task));
            }
        }
        None
    }

    fn handle_subcommand_matches(&mut self, matches: &ArgMatches) {
        match matches.subcommand() {
            Some((subcommand_name, sub_matches)) => {
                // Finde den entsprechenden Task
                if let Some((task_path, task)) = self.find_task_by_subcommand(subcommand_name) {
                    // Setze Argumente
                    let task_arg_names: Vec<String> = task.arguments.keys().cloned().collect();
                    
                    if let Some(task_mut) = self.tasks.get_mut_by_path(&task_path) {
                        for arg_name in task_arg_names {
                            if let Some(value) = sub_matches.get_one::<String>(&arg_name) {
                                task_mut.set_argument_value(&arg_name, value);
                                println!("Set argument '{}' = '{}'", arg_name, value);
                            }
                        }

                        println!("Executing task: {}", subcommand_name);
                        task_mut.run();
                    }
                } else {
                    println!("Task not found for subcommand: {}", subcommand_name);
                }
            }
            None => {
                println!("No subcommand specified. Use --help to see available tasks.");
            }
        }
    }
}
