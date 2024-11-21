package com.plugin.tcp

import android.util.Log

class TcpClient {
    fun pong(value: String): String {
        Log.i("Pong", value)
        return value
    }
}
