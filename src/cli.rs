use crate::server_state::Role;

pub struct CliArgs {
    pub host: Option<String>,
    pub port: Option<String>,
    pub role: Role,
}

impl CliArgs {
    pub fn parse() -> CliArgs {
        let mut port = None;
        let mut host = None;
        let mut role = Role::Master;

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--host" => {
                    host = args.next();
                }
                "--port" => {
                    port = args.next();
                }
                "--replicaof" => {
                    let host = args.next().unwrap();
                    let port = args.next().unwrap();
                    role = Role::Slave {
                        master_host: host,
                        master_port: port,
                    };
                }
                _ => {
                    panic!("unknown option: {}", arg);
                }
            }
        }
        CliArgs { host, port, role }
    }
}
