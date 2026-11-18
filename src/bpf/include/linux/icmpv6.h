#pragma once

#include <vmlinux.h>

#define ICMPV6_DEST_UNREACH 1
#define ICMPV6_PKT_TOOBIG 2
#define ICMPV6_TIME_EXCEED 3
#define ICMPV6_PARAMPROB 4
#define ICMPV6_ECHO_REQUEST 128
#define ICMPV6_ECHO_REPLY 129
#define ICMPV6_NOROUTE 0
#define ICMPV6_ADM_PROHIBITED 1
#define ICMPV6_NOT_NEIGHBOUR 2
#define ICMPV6_ADDR_UNREACH 3
#define ICMPV6_PORT_UNREACH 4
#define ICMPV6_EXC_HOPLIMIT 0
#define ICMPV6_EXC_FRAGTIME 1
#define ICMPV6_HDR_FIELD 0
#define ICMPV6_UNK_NEXTHDR 1

#define icmp6_pointer icmp6_dataun.un_data32[0]
#define icmp6_mtu icmp6_dataun.un_data32[0]
