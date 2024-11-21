use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_tcp_client);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<TcpClient<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("com.plugin.tcp", "TcpClientPlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_tcp_client)?;
  Ok(TcpClient(handle))
}

/// Access to the tcp-client APIs.
pub struct TcpClient<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> TcpClient<R> {
  pub fn connect(&self, payload: ConnectRequest) -> crate::Result<ConnectResponse> {
    self
      .0
      .run_mobile_plugin("connect", payload)
      .map_err(Into::into)
  }

  pub fn transmit(&self, payload: TransmitRequest) -> crate::Result<TransmitResponse> {
    self
      .0
      .run_mobile_plugin("transmit", payload)
      .map_err(Into::into)
  }

  pub fn disconnect(&self) -> crate::Result<()> {
    self
      .0
      .run_mobile_plugin("disconnect", ())
      .map_err(Into::into)
  }
}
