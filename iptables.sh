/sbin/iptables -t mangle -F

/sbin/ip rule add fwmark 0x1 lookup 100
/sbin/ip route add local 0.0.0.0/0 dev lo table 100


/sbin/iptables -t mangle -N DIVERT
/sbin/iptables -t mangle -A DIVERT -j MARK --set-mark 1
/sbin/iptables -t mangle -A DIVERT -j ACCEPT

/sbin/iptables -t mangle -A PREROUTING -p tcp -m socket  -j DIVERT

/sbin/iptables -t mangle -A PREROUTING -p tcp --dport 80 -i enp0s3 ! -d 192.168.2.192 -j TPROXY --tproxy-mark 0x1/0x1 --on-port 3128 --on-ip 192.168.2.192
