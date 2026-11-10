// what a fake client's getGameConnectionInfo writes: the connection type, the host and the port
#[derive(Debug, Clone)]
pub struct FakeConnection {
    pub connection_type: i16,
    pub host: String,
    pub port: i32,
}
