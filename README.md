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

## Connecting
```bash
telnet 172.16.10.194 4567
```

## Git workflow

Before starting work:
```bash
cd /home/william-test/chat-app
git pull
```

After making changes:
```bash
git add .
git commit -m "commit message"
git push
```

Creating a branch
```bash
git switch -c my-feature
```

Working on a branch:s
```bash
git add. 
git commit -m "commit message"
git push -u origin my-feature
git push
```
Switching between branches
```bash
git switch main
```

## To-do 
- Storage for chats (save/load)
- Better way to quit/leave chat
- UI
- Client to replace telnet
- Script to quickstart and stop the server
- CCTV 
- Thread-per-client model won't scale

