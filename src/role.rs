use crate::store;

const ROLE_KEY: &str = "__role__";

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

pub fn get_role() -> Role {
    match store::get(ROLE_KEY) {
        Some(role) => Role::from_string(&role),
        None => panic!("role not found"),
    }
}

pub fn set_role(role: Role) {
    store::set(ROLE_KEY, &role.to_string(), None);
}
