# TCP Commander

Small GUI application for sending TCP messages to a server.

> "Wait a minute, this is a web app!"  

Your observation is correct! I do not feel like learning Java/Kotlin to make an Android app!

## For Students

This section is directed towards students who are using this app - you guys know who you are.
If you're not a student, feel free to ignore this section.

> When connecting, I get "NoRouteToHostException; potential firewall issue"

The main reasons for this that I'm aware of are that
- There is an error with the network configuration, likely stemming from the firewall;
try `ping`ing the server (i.e. `ping 192.168.86.42`) from another device to see if the issue persists,
and try connecting again if the pings are succeeding
- The server isn't currently up or you've put in the wrong address (usually this leads to timeout but
for some reason it will throw `NoRouteToHostException` instead)

🏗️ Documentation WIP 🚧
