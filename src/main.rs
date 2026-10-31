use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, Ipv6Addr},
    ptr::{self, NonNull},
    str::FromStr,
    thread,
};

use crate::clat::ClatSkelBuilder;
use anyhow::Context;
use clap::Parser;
use libbpf_rs::{
    AsRawLibbpf, Link,
    skel::{OpenSkel, SkelBuilder},
};

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
        return Ok(Self {
            address: address_str.parse().map_err(|e| {
                anyhow::format_err!("invalid IPv6 address {:?}: {}", address_str, e)
            })?,
            prefix_length,
        });
    }
}

/// A Netkit-based stateless NAT64 implementation.
#[derive(clap::Parser)]
struct Args {
    /// Netkit primary interface to attach to.
    ///
    /// IPv6 packets sent to this interface will be translated to IPv4 before
    /// returning through its peer interface and vice versa.
    ///
    /// The MTU of the peer interface should be 20 bytes lower than its primary.
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
}

mod clat {
    include!(concat!(env!("OUT_DIR"), "/clat.skel.rs"));
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut open_skel = MaybeUninit::uninit();
    let skel = ClatSkelBuilder::default()
        .open(&mut open_skel)?
        .load()
        .context("unable to load BPF programs")?;

    let bss_data = skel.maps.bss_data.unwrap();
    bss_data.config.pref64.in6_u.u6_addr8 = args.prefix.address.octets();
    bss_data.config.pref64_len = args.prefix.prefix_length;
    bss_data.config.untranslatable_v4.s_addr = args.untranslatable_v4.to_bits().to_be();

    let ifindex = nix::net::if_::if_nametoindex(args.interface.as_str())
        .context("unable to get interface index")?;

    let _primary_link = unsafe {
        Link::from_ptr(
            NonNull::new(libbpf_rs::libbpf_sys::bpf_program__attach_netkit(
                skel.progs.nm_clat_primary.as_libbpf_object().as_ptr(),
                ifindex as i32,
                ptr::null(),
            ))
            .ok_or_else(|| anyhow::format_err!("unable to attach primary BPF program"))?,
        )
    };

    let _peer_link = unsafe {
        Link::from_ptr(
            NonNull::new(libbpf_rs::libbpf_sys::bpf_program__attach_netkit(
                skel.progs.nm_clat_peer.as_libbpf_object().as_ptr(),
                ifindex as i32,
                ptr::null(),
            ))
            .ok_or_else(|| anyhow::format_err!("unable to attach peer BPF program"))?,
        )
    };

    eprintln!("nkxlat: programs attached");

    loop {
        thread::park();
    }
}
