pub mod commands{
	use tui::{
		widgets::{ListState, ListItem},
		style::{Style},
	};
	use tokio::process::{
		Command as Cmd,
		Child,
		ChildStdout
	};
	use tokio::io::{BufReader, AsyncBufReadExt, Lines};
	use std::process::Stdio;
	use std::sync::Arc;
	use std::sync::Mutex;
	use crate::configuration::configuration::{
		Configuration,
		parse_configuration
	};

	#[derive(Debug)]
	pub struct Command<'a>{
		pub child_process: Option<Arc<Mutex<Child>>>,
		pub stdout: Option<Lines<BufReader<ChildStdout>>>,
		command: &'a str,
		arguments: Vec<&'a str>,
		pub output: Vec<String>
	}

	impl Command<'_>{
		fn new<'a>(config: &Configuration<'a>) -> Command<'a>{
			Command {
				child_process: None,
				stdout: None,
				command: config.command,
				arguments: config.arguments.clone(),
				output: vec!()
			}
		}

		async fn run(&mut self){
			let mut cmd = Cmd::new(self.command);
			cmd.stdout(Stdio::piped());
			cmd.args(&self.arguments);

			let mut child = cmd
				.spawn()
				.expect("Failed to execute command");

			let stdout = child.stdout.take()
				 .expect("child did not have a handle to stdout");

			let mut stdout = BufReader::new(stdout).lines();
			self.child_process = Some(Arc::new(Mutex::new(child)));

			while let Some(line) = stdout.next_line().await.unwrap() {
				self.output.push(line);
			}
		}

		async fn kill(&self){
			let child = self.child_process.clone();
			let child = child.unwrap();
			let mut child = child.lock().unwrap();

			match child.kill().await {
				Ok(()) => (),
				Err(e) => {
					if e.kind() != std::io::ErrorKind::InvalidInput {
						println!("{:?}", e.kind());
					}
				}
			}
		}
	}

	pub struct Commands<'a>{
		pub state: ListState,
		pub commands: Vec<Command<'a>>,
		pub items: Vec<ListItem<'a>>
	}

	impl Commands<'_>{
		pub fn new(config: &str) -> Commands {
			let values: Vec<Configuration> = parse_configuration(config);
			let commands: Vec<Command> = values.iter()
				.map(|v| {
					Command::new(v)
				})
				.collect();
			let items: Vec<ListItem> = commands.iter()
				.map(|command| {
					ListItem::new(command.command)
						.style(Style::default())
				})
				.collect();

			Commands {
				state: ListState::default(),
				commands: commands,
				items: items
			}
		}

		pub async fn run(&mut self){
			for command in &mut self.commands {
				command.run().await;
			}
		}

		pub async fn kill(&mut self){
			for command in &self.commands {
				command.kill().await;
			}
		}

		pub fn next(&mut self) {
			let i = match self.state.selected() {
				Some(i) => {
					if i >= self.commands.len() - 1 {
						0
					} else {
						i + 1
					}
				}
				None => 0,
			};
			self.state.select(Some(i));
		}

		pub fn previous(&mut self) {
			let i = match self.state.selected() {
				Some(i) => {
					if i == 0 {
						self.commands.len() - 1
					} else {
						i - 1
					}
				}
				None => 0,
			};
			self.state.select(Some(i));
		}

		pub fn unselect(&mut self) {
			self.state.select(None);
		}
	}
}