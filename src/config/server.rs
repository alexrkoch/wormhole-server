use std::env::var;
use tracing::info;

const HOST_ENV_VAR: &str = "WORMHOLE_HOST";
const PORT_ENV_VAR: &str = "WORMHOLE_PORT";
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;

pub fn get_host() -> String {
    if let Ok(host) = var(HOST_ENV_VAR) {
        info!("Using {} as the host since {} is set", host, HOST_ENV_VAR);
        host
    } else {
        info!("Using {} as the host", DEFAULT_HOST);
        DEFAULT_HOST.to_owned()
    }
}

pub fn get_port() -> u16 {
    if let Ok(port) = var(PORT_ENV_VAR) {
        info!("Using {} as the port since {} is set", port, PORT_ENV_VAR);
        port.parse().expect(format!("The environment variable {PORT_ENV_VAR} contains an invalid port, please fix or delete it").as_str())
    } else {
        info!("Using {} as the port", DEFAULT_PORT);
        DEFAULT_PORT
    }
}
