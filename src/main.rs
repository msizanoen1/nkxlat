use std::{
    fs::File,
    io::ErrorKind,
    mem::MaybeUninit,
    net::{Ipv4Addr, Ipv6Addr},
    os::fd::{AsFd, AsRawFd},
    ptr,
    str::FromStr,
};

use crate::clat::ClatSkelBuilder;
use anyhow::Context;
use clap::Parser;
use daemonize::Daemonize;
use libbpf_rs::{
    AsRawLibbpf, TC_INGRESS, TcHookBuilder,
    libbpf_sys::bpf_program__attach_netkit,
    skel::{OpenSkel, SkelBuilder},
};
use netlink_bindings::{
    rt_link,
    traits::NetlinkRequest,
    utils::{Rec, finalize_nested_header, push_nested_header},
};
use netlink_socket2::{MulticastSocketRaw, NetlinkSocket};
use nix::{
    libc::{IFLA_INFO_DATA, RTM_DELLINK, RTNLGRP_LINK},
    net::if_::if_nametoindex,
    sched::{CloneFlags, setns, unshare},
};
use sd_notify::NotifyState;

#[derive(Clone, Copy, Debug)]
struct Pref64 {
    address: Ipv6Addr,
    prefix_length: u32,
}

impl FromStr for Pref64 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (address_str, prefix_length_str) = s
            .split_once("/")
            .ok_or_else(|| anyhow::format_err!("must be <IPv6 address>/<prefix length>"))?;
        let prefix_length = prefix_length_str.parse().map_err(|e| {
            anyhow::format_err!("invalid prefix length {:?}: {}", prefix_length_str, e)
        })?;
        match prefix_length {
            96 | 64 | 56 | 48 | 40 | 32 => (),
            _ => anyhow::bail!("unsupported prefix length {}", prefix_length),
        }
        Ok(Self {
            address: address_str.parse().map_err(|e| {
                anyhow::format_err!("invalid IPv6 address {:?}: {}", address_str, e)
            })?,
            prefix_length,
        })
    }
}

/// A Netkit-based stateless NAT64 implementation.
#[derive(clap::Parser)]
struct Args {
    /// Name of interface to create.
    ///
    /// IPv6 packets sent to this interface will be translated to IPv4 before
    /// returning through the same interface and vice versa.
    interface: String,
    /// NAT64 prefix to use for translation.
    ///
    /// Packet source and destination addresses will be translated according to
    /// RFC 6052 section 2.3 with the chosen prefix.
    ///
    /// Prefix lengths of 96, 64, 56, 48, 40 and 32 are supported.
    prefix: Pref64,
    /// IPv4 address to use for translating "untranslatable" ICMPv6 error
    /// packets as defined by draft-ietf-v6ops-icmpext-xlat-v6only-source.
    ///
    /// Certain ICMPv6 error packets with a source address outside the NAT64
    /// prefix will have its source address remapped to this value.
    #[arg(long, short, default_value = "192.0.0.8")]
    untranslatable_v4: Ipv4Addr,
    /// Don't daemonize.
    #[arg(long, short)]
    foreground: bool,
    /// Use veth even if netkit is supported by the kernel.
    #[arg(long, short)]
    veth: bool,
}

mod clat {
    include!(concat!(env!("OUT_DIR"), "/clat.skel.rs"));
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse();

    let system_netns =
        File::open("/proc/self/ns/net").context("unable to open current network namespace")?;

    unshare(CloneFlags::CLONE_NEWNET).context("unable to create network namespace")?;

    let mut multicast_sock = MulticastSocketRaw::new(rt_link::PROTONUM)
        .context("unable to create netlink multicast socket")?;
    multicast_sock
        .listen(RTNLGRP_LINK)
        .context("unable to bind netlink multicast socket")?;

    if !args.veth {
        let mut request = rt_link::Request::new()
            .set_create()
            .op_newlink_do(&rt_link::Ifinfomsg {
                ifi_flags: rt_link::IfinfoFlags::Up as _,
                ifi_change: rt_link::IfinfoFlags::Up as _,
                ..Default::default()
            });
        let mut encoder = request
            .encode()
            .push_ifname_bytes(args.interface.as_bytes())
            .nested_linkinfo()
            .nested_data_netkit()
            .push_peer_policy(rt_link::NetkitPolicy::Blackhole as _);
        let peer_info_offset = push_nested_header(encoder.as_vec_mut(), 1);
        encoder
            .as_vec_mut()
            .extend_from_slice(rt_link::Ifinfomsg::new().as_slice());
        rt_link::PushLinkAttrs::new(encoder.as_vec_mut())
            .push_net_ns_fd(system_netns.as_raw_fd() as _)
            .push_ifname_bytes(args.interface.as_bytes());
        finalize_nested_header(encoder.as_vec_mut(), peer_info_offset);
        drop(encoder);

        if let Err(e) = NetlinkSocket::new()
            .request(&request)
            .context("unable to send create interface request")?
            .recv_ack()
        {
            if e.as_io_error().kind() == ErrorKind::Unsupported {
                eprintln!("netkit not supported on this kernel, falling back to veth");
                args.veth = true;
            } else {
                return Err(e).context("unable to create interface");
            }
        }
    }

    if args.veth {
        let mut request = rt_link::Request::new()
            .set_create()
            .op_newlink_do(&rt_link::Ifinfomsg {
                ifi_flags: rt_link::IfinfoFlags::NoArp as u32 | rt_link::IfinfoFlags::Up as u32,
                ifi_change: rt_link::IfinfoFlags::NoArp as u32 | rt_link::IfinfoFlags::Up as u32,
                ..Default::default()
            });
        let mut encoder = request
            .encode()
            .push_ifname_bytes(args.interface.as_bytes())
            .nested_linkinfo()
            .push_kind_bytes(b"veth");
        let data_offset = push_nested_header(encoder.as_vec_mut(), IFLA_INFO_DATA);
        let peer_info_offset = push_nested_header(encoder.as_vec_mut(), 1);
        encoder.as_vec_mut().extend_from_slice(
            rt_link::Ifinfomsg {
                ifi_flags: rt_link::IfinfoFlags::NoArp as _,
                ifi_change: rt_link::IfinfoFlags::NoArp as _,
                ..Default::default()
            }
            .as_slice(),
        );
        rt_link::PushLinkAttrs::new(encoder.as_vec_mut())
            .push_net_ns_fd(system_netns.as_raw_fd() as _)
            .push_ifname_bytes(args.interface.as_bytes());
        finalize_nested_header(encoder.as_vec_mut(), peer_info_offset);
        finalize_nested_header(encoder.as_vec_mut(), data_offset);
        drop(encoder);

        NetlinkSocket::new()
            .request(&request)
            .context("unable to send create interface request")?
            .recv_ack()
            .context("unable to create interface")?;
    }

    let ifindex =
        if_nametoindex(args.interface.as_str()).context("unable to get interface index")? as i32;

    let mut open_skel = MaybeUninit::uninit();
    let skel = ClatSkelBuilder::default()
        .open(&mut open_skel)
        .context("unable to open BPF program")?
        .load()
        .context("unable to load BPF program")?;

    let bss_data = skel.maps.bss_data.unwrap();
    bss_data.config.pref64.in6_u.u6_addr8 = args.prefix.address.octets();
    bss_data.config.pref64_len = args.prefix.prefix_length;
    bss_data.config.untranslatable_v4.s_addr = args.untranslatable_v4.to_bits().to_be();

    if args.veth {
        TcHookBuilder::new(skel.progs.nm_clat_peer.as_fd())
            .ifindex(ifindex)
            .hook(TC_INGRESS)
            .handle(1)
            .priority(1)
            .create()
            .context("unable to create tc hook")?
            .attach()
            .context("unable to attach BPF program")?;
    } else {
        unsafe {
            if bpf_program__attach_netkit(
                skel.progs.nm_clat_peer.as_libbpf_object().as_ptr(),
                ifindex,
                ptr::null(),
            )
            .is_null()
            {
                anyhow::bail!("unable to attach BPF program");
            }
        }
    }

    if args.foreground {
        let nkxlat_netns =
            File::open("/proc/self/ns/net").context("failed to open created network namespace")?;

        setns(system_netns, CloneFlags::CLONE_NEWNET)
            .context("failed to switch to system network namespace")?;

        let _ = sd_notify::notify(&[NotifyState::Ready]);

        setns(nkxlat_netns, CloneFlags::CLONE_NEWNET)
            .context("failed to switch to created network namespace")?;
    } else {
        drop(system_netns);
        Daemonize::new().start().context("failed to daemonize")?;
    }

    loop {
        let (recv, buf) = multicast_sock
            .recv()
            .context("unable to read from netlink multicast socket")?;
        if recv.message_type == RTM_DELLINK {
            let (header, _attrs) = rt_link::OpGetlinkDo::decode_reply(buf);
            if header.ifi_index == ifindex {
                break;
            }
        }
    }

    Ok(())
}
