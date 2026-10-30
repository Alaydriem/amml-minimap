// the connection a world is played over, as the game's connection info names it
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerAddress {
    // the game's Social::ConnectionType: -1 undefined, 0 local, higher values are network connections
    pub connection_type: i16,
    pub host: String,
    pub port: u16,
}

impl ServerAddress {
    pub const UNDEFINED: i16 = -1;
    pub const LOCAL: i16 = 0;

    // a connection to another machine, named by a host
    pub fn is_remote(&self) -> bool {
        self.connection_type > Self::LOCAL && !self.host.is_empty()
    }
}
