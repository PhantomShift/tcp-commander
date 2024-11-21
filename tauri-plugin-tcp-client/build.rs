const COMMANDS: &[&str] = &["ping", "connect", "transmit", "disconnect"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
