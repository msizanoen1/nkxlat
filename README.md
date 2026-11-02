```
A Netkit-based stateless NAT64 implementation

Usage: nkxlat [OPTIONS] <INTERFACE> <PREFIX>

Arguments:
  <INTERFACE>
          Name of interface to create.
          
          IPv6 packets sent to this interface will be translated to IPv4 before returning through the same interface and vice versa.

  <PREFIX>
          NAT64 prefix to use for translation.
          
          Packet source and destination addresses will be translated according to RFC 6052 section 2.3 with the chosen prefix.
          
          Prefix lengths of 96, 64, 56, 48, 40 and 32 are supported.

Options:
  -u, --untranslatable-v4 <UNTRANSLATABLE_V4>
          IPv4 address to use for translating "untranslatable" ICMPv6 error packets as defined by draft-ietf-v6ops-icmpext-xlat-v6only-source.
          
          Certain ICMPv6 error packets with a source address outside the NAT64 prefix will have its source address remapped to this value.
          
          [default: 192.0.0.8]

  -f, --foreground
          Don't daemonize

  -v, --veth
          Use veth even if netkit is supported by the kernel

  -h, --help
          Print help (see a summary with '-h')
```

# Stateful NAT64 example

While this implementation does not provide a stateful NAT64, it can be combined with Linux's own stateful NAT44 and NAT66 to achieve a similar effect:

```sh
nkxlat nkxlat0 64:ff9b::/96
ip link set nkxlat0 up
ip address add 64:ff9b::192.0.0.7/96 dev nkxlat0
ip route add 192.0.0.7 dev nkxlat0
nft -f - <<EOF
destroy table inet nkxlat
table inet nkxlat {
    chain postrouting {
        type nat hook postrouting priority srcnat
        iifname nkxlat0 masquerade
        oifname nkxlat0 masquerade
    }
}
EOF
sysctl -wq net.ipv4.ip_forward=1
sysctl -wq net.ipv6.conf.all.forwarding=1
ping 64:ff9b::8.8.8.8
```

# Credits

The eBPF packet translation logic is derived from [NetworkManager's eBPF-based CLAT implementation](https://gitlab.freedesktop.org/NetworkManager/NetworkManager/-/blob/main/src/core/bpf/clat.bpf.c). The modified version used by this project can be found [here](https://github.com/msizanoen1/NetworkManager/blob/nkxlat/src/core/bpf/clat.bpf.c).
