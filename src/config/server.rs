use std::{borrow::Cow, env::var};
use tracing::info;

const HOST_ENV_VAR: &str = "WORMHOLE_HOST";
const PORT_ENV_VAR: &str = "WORMHOLE_PORT";
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;

pub fn get_host() -> Cow<'static, str> {
    match var(HOST_ENV_VAR) {
        Ok(host) => {
            info!("Using {} as the host since {} is set", host, HOST_ENV_VAR);
            host.into()
        }
        _ => {
            info!("Using {} as the host", DEFAULT_HOST);
            DEFAULT_HOST.into()
        }
    }
}

pub fn get_port() -> u16 {
    match var(PORT_ENV_VAR) {
        Ok(port) => {
            info!("Using {} as the port since {} is set", port, PORT_ENV_VAR);
            port.parse().expect(format!("The environment variable {PORT_ENV_VAR} contains an invalid port, please fix or delete it").as_str())
        }
        _ => {
            info!("Using {} as the port", DEFAULT_PORT);
            DEFAULT_PORT
        }
    }
}
