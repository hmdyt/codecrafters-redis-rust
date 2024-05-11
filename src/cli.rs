pub struct CliArgs {
    pub host: Option<String>,
    pub port: Option<String>,
}

impl CliArgs {
    pub fn parse() -> CliArgs {
        let mut port = None;
        let mut host = None;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--host" => {
                    host = args.next();
                }
                "--port" => {
                    port = args.next();
                }
                _ => {
                    panic!("unknown option: {}", arg);
                }
            }
        }
        CliArgs { host, port }
    }
}
