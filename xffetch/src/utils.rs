pub enum OS {
    Linux,
    FreeBSD,
    OpenBSD,
    NetBSD,
    DragonflyBSD,
    Other,
}

pub struct OSInfo {
    pub os: OS,
    pub version: String,
}

impl OSInfo {
    pub fn new() -> OSInfo {
        let os = OS::Other;
        let version = String::from("Unknown");
        OSInfo {
            os: os,
            version: version,
        }
    }
}

