#ifndef _SYS_SOCKET_H
#define _SYS_SOCKET_H 1

typedef unsigned short sa_family_t;

struct sockaddr {
    sa_family_t sa_family;
    char sa_data[14];
};

#endif
