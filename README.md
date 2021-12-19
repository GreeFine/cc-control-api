# CC-API 

This project is an API to communicate and order arround ComputerCraft turtle in Minecraft

# Start

```bash
RUST_LOG=cc_api cargo run
```

# The lua files that is used by the turtle:
[main.lua](./main.lua) This file is served by the server, making it easyer to keep the turtle updated
```bash
curl -X GET localhost:8787/luafile
```

# Requests

## Send orders

Examples:

Move arround
```bash
curl -X POST -d $'orders=Forward,3\nLeft,1\nForward,5\nDown,4' -H "application/json"  localhost:8787/order/NameOfYourTurtle
```

Go to home
```bash
curl -X POST -d $'orders=Home,0' -H "application/json" -v localhost:8787/order/NameOfYourTurtle
```

Reboot
```bash
curl -X POST -d $'orders=Reboot,0' -H "application/json" -v localhost:8787/order/NameOfYourTurtle
```
<hr/>

## Get Informations

Get position
```bash
curl -X GET localhost:8787/pos/NameOfYourTurtle
```

Get info on a topic
```bash
curl -X GET localhost:8787/info/NameOfYourTurtle/YourTopic
# curl -X GET localhost:8787/info/Kubernetes/fuelLevel
```