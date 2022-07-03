pub enum Command {
    Serve,
    Client,
}

impl Into<Command> for String {
    fn into(self) -> Command {
        match self.as_str() {
            "serve" => {
                Command::Serve
            }
            "client" => {
                Command::Client
            }
            _ => {
                panic!("Unrecognized command `{self}'")
            }
        }
    }
}