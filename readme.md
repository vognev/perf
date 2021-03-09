### perf

`perf` just counts bytes it is receiving over tcp or udp and calculating transfer rate

### usage

#### tcp

`./perf -m tcp`

`socat -b 1200 - tcp:<peer_ip>:<peer_port> </dev/zero`

#### udp

`./perf -m udp`

`socat -b 1200 - udp:<peer_ip>:<peer_port> </dev/zero`
