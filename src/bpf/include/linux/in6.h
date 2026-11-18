#pragma once

#include <vmlinux.h>

#define IPPROTO_FRAGMENT 44 /* IPv6 fragmentation header	*/
#define IPPROTO_ICMPV6 58   /* ICMPv6			*/

#define s6_addr in6_u.u6_addr8
#define s6_addr16 in6_u.u6_addr16
#define s6_addr32 in6_u.u6_addr32
