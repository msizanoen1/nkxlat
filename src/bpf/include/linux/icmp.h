#pragma once

#include <vmlinux.h>

#define ICMP_ECHOREPLY 0      /* Echo Reply			*/
#define ICMP_DEST_UNREACH 3   /* Destination Unreachable	*/
#define ICMP_ECHO 8           /* Echo Request			*/
#define ICMP_TIME_EXCEEDED 11 /* Time Exceeded		*/
#define ICMP_PARAMETERPROB 12 /* Parameter Problem		*/
#define ICMP_NET_UNREACH 0    /* Network Unreachable		*/
#define ICMP_HOST_UNREACH 1   /* Host Unreachable		*/
#define ICMP_PROT_UNREACH 2   /* Protocol Unreachable		*/
#define ICMP_PORT_UNREACH 3   /* Port Unreachable		*/
#define ICMP_FRAG_NEEDED 4    /* Fragmentation Needed/DF set	*/
#define ICMP_SR_FAILED 5      /* Source Route failed		*/
#define ICMP_NET_UNKNOWN 6
#define ICMP_HOST_UNKNOWN 7
#define ICMP_HOST_ISOLATED 8
#define ICMP_NET_ANO 9
#define ICMP_HOST_ANO 10
#define ICMP_NET_UNR_TOS 11
#define ICMP_HOST_UNR_TOS 12
#define ICMP_PKT_FILTERED 13 /* Packet filtered */
#define ICMP_PREC_CUTOFF 15  /* Precedence cut off */
#define ICMP_EXC_TTL 0       /* TTL count exceeded		*/
#define ICMP_EXC_FRAGTIME 1  /* Fragment Reass time exceeded	*/

#define ICMP_AFI_IP6 2 /* Address Family Identifier for ipv6 */
