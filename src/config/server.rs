//! Values and utility functions for configuring the server

use std::{borrow::Cow, env::var};
use tracing::info;

/// The name of the environment variable that can be used to override the host
const HOST_ENV_VAR: &str = "WORMHOLE_HOST";
/// The name of the environment variable that can be used to override the port
const PORT_ENV_VAR: &str = "WORMHOLE_PORT";
/// The default host that the server is configured to use
const DEFAULT_HOST: &str = "127.0.0.1";
/// The default port that the server is configured to use
const DEFAULT_PORT: u16 = 8080;

/// Gets the server host name.
/// Reads from the [environment][HOST_ENV_VAR] if available, otherwise falls back to the [default
/// value][DEFAULT_HOST]
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

/// Gets the server port.
/// Reads from the [environment][PORT_ENV_VAR] if available, otherwise falls back to the [default
/// value][DEFAULT_PORT]
pub fn get_port() -> u16 {
    match var(PORT_ENV_VAR) {
        Ok(port) => {
            info!("Using {} as the port since {} is set", port, PORT_ENV_VAR);
            port.parse().unwrap_or_else(|_| panic!("The environment variable {PORT_ENV_VAR} contains an invalid port, please fix or delete it"))
        }
        _ => {
            info!("Using {} as the port", DEFAULT_PORT);
            DEFAULT_PORT
        }
    }
}
