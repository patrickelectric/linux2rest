# linux2rest

[![Deploy unix](https://github.com/patrickelectric/linux2rest/actions/workflows/build.yml/badge.svg)](https://github.com/patrickelectric/linux2rest/actions/workflows/build.yml)

Provides computer information through a REST API.

Simple website available in: `localhost:6030`
REST API documentation in: `localhost:6030/docs`

Features:
- Provides real time kernel messages via websocket
- Netstat information
- Platform specific information (Raspberry: undervoltage, cpu throttling and etc)
- System information
  - CPU
  - Disk
  - OS info
  - Memory
  - Network
  - Processes (pid, user, cpu usage, memory, path, uptime..., like htop)
  - Sensors (Temperature)
  - Current unix time
- Udev tree information