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
                    // 1個目の引数が空白でsplitできる場合 -> --replicaof "host port"
                    // できない場合 -> --replicaof host port
                    let first = args.next().unwrap();
                    if first.contains(' ') {
                        let mut split = first.split(' ');
                        let host = split.next().unwrap();
                        let port = split.next().unwrap();
                        role = Role::Slave {
                            master_host: host.to_string(),
                            master_port: port.to_string(),
                        };
                    } else {
                        let host = first;
                        let port = args.next().unwrap();
                        role = Role::Slave {
                            master_host: host,
                            master_port: port,
                        };
                    }
                }
                _ => {
                    panic!("unknown option: {}", arg);
                }
            }
        }
        CliArgs { host, port, role }
    }
}
