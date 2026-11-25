#!/bin/bash

firewall-cmd --add-port=4567/tcp
telnet 172.16.10.185 4567
