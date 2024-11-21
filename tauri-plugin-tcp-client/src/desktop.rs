use std::{
    borrow::BorrowMut, io::Write, net::{SocketAddr, TcpStream}, sync::Mutex
};

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<TcpClient<R>> {
    Ok(TcpClient(app.clone(), Mutex::new(PluginState::default())))
}

/// Access to the tcp-client APIs.
pub struct TcpClient<R: Runtime>(AppHandle<R>, Mutex<PluginState>);

impl<R: Runtime> TcpClient<R> {
    pub fn connect(&self, payload: ConnectRequest) -> crate::Result<ConnectResponse> {
        let mut state = self.1.lock().unwrap();
        if let Some(socket) = state.active_socket.as_ref() {
            socket.shutdown(std::net::Shutdown::Write)?;
            state.active_socket = None;
        }

        let addr = payload.address.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing address")
        })?;
        let port = payload
            .port
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing port"))?;

        let socket = TcpStream::connect(SocketAddr::new(
            addr.parse().expect("invalid ip address"),
            port as u16,
        ));

        match socket {
            Ok(socket) => {
                state.active_socket = Some(socket);
                Ok(ConnectResponse {
                    success: Some(true),
                    error: None,
                })
            }
            Err(e) => {
                Ok(ConnectResponse {
                    success: Some(false),
                    error: Some(format!("Failed to connect to server: {e}")),
                })
            }
        }
    }

    pub fn disconnect(&self) -> crate::Result<()> {
        let mut  state = self.1.lock().unwrap();
        if let Some(socket) = &state.active_socket {
            socket.shutdown(std::net::Shutdown::Write).expect("Failed to shut down socket");
            state.active_socket = None;
        }

        Ok(())
    }

    pub fn transmit(&self, payload: TransmitRequest) -> crate::Result<TransmitResponse> {
        let mut state = self.1.lock().unwrap();
        if let Some(ref mut socket) = state.active_socket.borrow_mut() {
            if let Err(_e) = socket.write_all(payload.message.expect("missing message").as_bytes()) {
                let _ = socket.shutdown(std::net::Shutdown::Write);
                state.active_socket = None;
                Ok(TransmitResponse { error: Some("Error transmitting: disconnected from server".into()) })
            } else {
                Ok(TransmitResponse { error: None })
            }
        } else {
            Ok(TransmitResponse { error: Some("Error transmitting: not connected to a server".into()) })
        }
    }
}
