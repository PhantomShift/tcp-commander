package com.plugin.tcp

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import kotlinx.coroutines.sync.Mutex
import java.io.IOException
import java.net.ConnectException
import java.net.InetSocketAddress
import java.net.NoRouteToHostException
import java.net.Socket
import java.net.SocketAddress
import java.net.SocketTimeoutException

@InvokeArg
class PingArgs {
  var value: String? = null
}

@InvokeArg
class ConnectArgs {
    val address: String? = null
    val port: Int? = null
}

@InvokeArg
class TransmitArgs {
    val message: String? = null
}

@TauriPlugin
class TcpClientPlugin(private val activity: Activity): Plugin(activity) {
    private val implementation = TcpClient()
    private var active_socket: Socket? = null
    private var current_address: String? = null
    private var current_port: Int? = null
    private var connecting: Boolean = false

    @Command
    fun ping(invoke: Invoke) {
        val args = invoke.parseArgs(PingArgs::class.java)

        val ret = JSObject()
        ret.put("value", implementation.pong(args.value ?: "default value :("))
        invoke.resolve(ret)
    }

    @Command
    fun connect(invoke: Invoke) {
        if (connecting) {
            val ret = JSObject()
            ret.put("error", "Currently attempting to connect; please wait for success or error")

            invoke.resolve(ret)
            return;
        }
        connecting = true

        val args: ConnectArgs = invoke.parseArgs(ConnectArgs::class.java)
        requireNotNull(args.address) { "Missing address" }
        requireNotNull(args.port) { "Missing port number" }

        Thread({
            val ret = JSObject()
            try {
                if (active_socket == null
                    || current_address != args.address
                    || current_port != args.port) {
                    active_socket?.close()
                    active_socket = Socket(args.address, args.port)
                    active_socket.setReuseAddress(true);
                    current_address = args.address
                    current_port = args.port
                }
                ret.put("success", true)
            } catch (e: ConnectException) {
                println(e)
                ret.put("error", e.toString())
            } catch (e: SocketTimeoutException) {
                println(e)
                ret.put("error", e.toString())
            } catch (e: NoRouteToHostException) {
                println(e);
                ret.put("error", "NoRouteToHostException; potential firewall issue")
            }
            connecting = false
            invoke.resolve(ret)
        }).start()
    }

    @Command
    fun disconnect(invoke: Invoke) {
        if (connecting) {
            invoke.resolve()
            return
        }

        active_socket?.close()
        active_socket = null
        invoke.resolve()
    }

    @Command
    fun transmit(invoke: Invoke) {
        val ret = JSObject()

        if (connecting) {
            ret.put("error", "Currently attempting to connect, please wait")
            invoke.resolve(ret)
            return
        }

        if (active_socket?.isConnected != true) {
            ret.put("error", "Error transmitting: not connected to server")
            invoke.resolve(ret)
            return
        }
        requireNotNull(active_socket) { "Not connected" }

        val args: TransmitArgs = invoke.parseArgs(TransmitArgs::class.java)
        requireNotNull(args.message) { "Missing message" }

        Thread({
            try {
                println("Transmitting message: " + args.message)
                val bytes = args.message.toByteArray()
                val outputStream = active_socket!!.getOutputStream()
                outputStream.write(bytes)
            } catch (e: IOException) {
                ret.put("error", "Error transmitting: disconnected from server; please reconnect")
                active_socket!!.close()
                active_socket = null
            }

            invoke.resolve(ret)
        }).start()
    }

    @Command
    fun get_status(invoke: Invoke) {
        val msg = when (active_socket) {
            null -> "no socket"
            else -> if (active_socket!!.isConnected) "connected" else "disconnected"
        }

        val ret = JSObject()
        ret.put("value", msg)
        invoke.resolve(ret)
    }
}
