pub struct DelugeConnectionInfo {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl DelugeConnectionInfo {
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
        }
    }
}
