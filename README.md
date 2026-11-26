# VM-to-VM Communication

A simple chat server for sending messages between VMs.

## Setup

### Install dependencies
```bash
sudo dnf install -y git
sudo dnf install -y telnet
```

### Firewall
Open the port:
```bash
firewall-cmd --add-port=4567/tcp
```

Close the port:
```bash
firewall-cmd --remove-port=4567/tcp
```

## Connecting with WebSocket (New)
```bash
cargo run --bin chat-client
```

## Connecting with TCP (old)
```bash
telnet 172.16.10.194 4567
```

## To-do 
- Storage for chats (save/load)
- Better way to quit/leave chat
- UI
- Client to replace telnet
- Script to quickstart and stop the server
- CCTV 
- Thread-per-client model won't scale

