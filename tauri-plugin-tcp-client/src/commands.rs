use tauri::{command, Runtime};

use crate::models::*;
use crate::Result;
use crate::TcpClientExt;

#[command]
pub(crate) async fn connect<R: Runtime>(
    app: tauri::AppHandle<R>, 
    address: Option<String>,
    port: Option<i32>
) -> Result<ConnectResponse> {
    app.tcp_client().connect(ConnectRequest { address, port })
}

#[command]
pub(crate) async fn disconnect<R: Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
  app.tcp_client().disconnect()
}

#[command]
pub(crate) async fn transmit<R: Runtime>(
    app: tauri::AppHandle<R>, 
    message: Option<String>,
) -> Result<TransmitResponse> {
    app.tcp_client().transmit(TransmitRequest { message })
}
