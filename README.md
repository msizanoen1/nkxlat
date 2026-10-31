```
A Netkit-based stateless NAT64 implementation

Usage: nkxlat [OPTIONS] <INTERFACE> <PREFIX>

Arguments:
  <INTERFACE>
          Netkit primary interface to attach to.
          
          IPv6 packets sent to this interface will be translated to IPv4 before returning through its peer interface and vice versa.
          
          The MTU of the peer interface should be 20 bytes lower than its primary.

  <PREFIX>
          NAT64 prefix to use for translation.
          
          Packet source and destination addresses will be translated according to RFC 6052 section 2.3 with the chosen prefix.
          
          Prefix lengths of 96, 64, 56, 48, 40 and 32 are supported.

Options:
  -u, --untranslatable-v4 <UNTRANSLATABLE_V4>
          IPv4 address to use for translating "untranslatable" ICMPv6 error packets as defined by draft-ietf-v6ops-icmpext-xlat-v6only-source.
          
          Certain ICMPv6 error packets with a source address outside the NAT64 prefix will have its source address remapped to this value.
          
          [default: 192.0.0.8]

  -h, --help
          Print help (see a summary with '-h')
```

# Stateful NAT64 example

While this implementation does not provide a stateful NAT64, it can be combined with Linux's own stateful NAT44 and NAT66 to achieve a similar effect:

```sh
ip link add nat64 mtu 1500 type netkit blackhole peer nat46 mtu 1480
ip link set nat64 up
ip link set nat46 up
ip route add 64:ff9b::/96 dev nat64
ip route add 192.168.64.1 dev nat46
nft -f - <<EOF
destroy table inet nat64
table inet nat64 {
    chain postrouting {
        type nat hook postrouting priority srcnat
        iifname != nat64 oifname nat64 snat ip6 to 64:ff9b::192.168.64.1
        iifname nat46 oifname != nat46 masquerade
    }
}
EOF
sysctl -wq net.ipv4.ip_forward=1
sysctl -wq net.ipv6.conf.all.forwarding=1
nkxlat nat64 64:ff9b::/96
```

In another terminal:
```sh
ping 64:ff9b::8.8.8.8
```
