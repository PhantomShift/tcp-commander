# Tauri Plugin tcp-client

TCP client plugin written specifically for this project, in particular to support Android.
[This plugin](https://github.com/kuyoonjo/tauri-plugin-tcp) only supports desktop platforms
and the server capability is unnecessary, so this was written as a project-specific alternative.

It is essentially just a very thin wrapper around a Java class that manages a TCP socket on
Android and a static state on desktop.
