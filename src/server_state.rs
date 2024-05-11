use core::str;

use crate::store;

const ROLE_KEY: &str = "__role__";
const MASTER_REPLID_KEY: &str = "__master_replid__";
const MASTER_REPL_OFFSET_KEY: &str = "__master_repl_offset__";

pub enum Role {
    Master,
    Slave {
        master_host: String,
        master_port: String,
    },
}

// masterなら "master"
// slaveなら "slave,host,port"
impl Role {
    fn from_string(role: &str) -> Self {
        if role == "master" {
            Role::Master
        } else if role.starts_with("slave") {
            let mut iter = role.split(',');
            iter.next(); // consume "slave"
            let master_host = iter.next().unwrap().to_string();
            let master_port = iter.next().unwrap().to_string();
            Role::Slave {
                master_host,
                master_port,
            }
        } else {
            panic!("unknown role");
        }
    }

    fn to_string(&self) -> String {
        match self {
            Role::Master => "master".to_string(),
            Role::Slave {
                master_host,
                master_port,
            } => format!("slave,{},{}", master_host, master_port),
        }
    }
}

pub struct ServerState {
    pub role: Role,
    pub master_replid: String,
    pub master_repl_offset: u64,
}

impl ServerState {
    pub fn init(role: Role) {
        Self::set(ServerState {
            role,
            master_replid: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string(),
            master_repl_offset: 0,
        });
    }

    pub fn set(s: Self) {
        store::set(ROLE_KEY, &s.role.to_string(), None);
        store::set(MASTER_REPLID_KEY, &s.master_replid, None);
        store::set(
            MASTER_REPL_OFFSET_KEY,
            &s.master_repl_offset.to_string(),
            None,
        );
    }

    pub fn get() -> Self {
        let role = match store::get(ROLE_KEY) {
            Some(role) => Role::from_string(&role),
            None => panic!("role not found"),
        };
        let master_replid = match store::get(MASTER_REPLID_KEY) {
            Some(master_replid) => master_replid,
            None => panic!("master_replid not found"),
        };
        let master_repl_offset = match store::get(MASTER_REPL_OFFSET_KEY) {
            Some(master_repl_offset) => master_repl_offset.parse().unwrap(),
            None => panic!("master_repl_offset not found"),
        };
        ServerState {
            role,
            master_replid,
            master_repl_offset,
        }
    }
}
