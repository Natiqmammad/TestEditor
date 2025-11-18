use std::collections::{HashMap, HashSet};
use std::net::{Ipv4Addr, ToSocketAddrs};

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::support::{expect_int_arg, expect_string_arg};
use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($map:expr, $name:literal, $func:ident) => {
            $map.insert(
                $name.to_string(),
                NativeCallable::new(concat!("net::", $name), $func),
            );
        };
    }

    add!(&mut functions, "resolve_host", resolve_host);
    add!(&mut functions, "parse_ipv4", parse_ipv4);
    add!(&mut functions, "subnet_mask", subnet_mask);
    add!(&mut functions, "is_private_ipv4", is_private_ipv4);
    add!(&mut functions, "cidr_contains", cidr_contains);
    add!(&mut functions, "cidr_overlap", cidr_overlap);
    add!(&mut functions, "ipv4_network", ipv4_network);
    add!(&mut functions, "ipv4_broadcast", ipv4_broadcast);
    add!(&mut functions, "ipv4_range", ipv4_range);
    add!(&mut functions, "ipv4_same_subnet", ipv4_same_subnet);
    add!(&mut functions, "ipv4_to_int", ipv4_to_int);
    add!(&mut functions, "int_to_ipv4", int_to_ipv4);
    add!(&mut functions, "ipv4_class", ipv4_class);
    add!(&mut functions, "is_loopback", is_loopback);
    add!(&mut functions, "ipv4_next", ipv4_next);
    add!(&mut functions, "ipv4_prev", ipv4_prev);
    add!(&mut functions, "ipv4_host_count", ipv4_host_count);
    add!(&mut functions, "reverse_ptr", reverse_ptr);
    add!(&mut functions, "is_multicast", is_multicast);
    add!(&mut functions, "mask_to_prefix", mask_to_prefix);
    add!(&mut functions, "ipv4_to_binary", ipv4_to_binary);
    add!(&mut functions, "is_link_local", is_link_local);
    add!(&mut functions, "ipv4_supernet", ipv4_supernet);
    add!(&mut functions, "cidr_split", cidr_split);
    registry.register_module("net", functions);
}

fn resolve_host(args: &[Value]) -> Result<Value, ApexError> {
    let host = expect_string_arg(args, 0, "net.resolve_host")?;
    let lookup_target = format!("{}:0", host);
    let mut set = HashSet::new();
    for addr in lookup_target
        .to_socket_addrs()
        .map_err(|err| ApexError::new(format!("Failed to resolve '{}': {}", host, err)))?
    {
        set.insert(addr.ip().to_string());
    }
    Ok(Value::Tuple(
        set.into_iter().map(Value::String).collect::<Vec<_>>(),
    ))
}

fn parse_ipv4(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.parse_ipv4")?;
    Ok(Value::Bool(addr.parse::<Ipv4Addr>().is_ok()))
}

fn subnet_mask(args: &[Value]) -> Result<Value, ApexError> {
    let bits = expect_int_arg(args, 0, "net.subnet_mask")?;
    let bits = bits.to_u32().ok_or_else(|| {
        ApexError::new("net.subnet_mask expects a prefix length between 0 and 32")
    })?;
    if bits > 32 {
        return Err(ApexError::new(
            "net.subnet_mask expects a prefix length between 0 and 32",
        ));
    }
    let mask = if bits == 0 {
        0
    } else {
        u32::MAX << (32 - bits)
    };
    let octets = mask.to_be_bytes();
    Ok(Value::String(format!(
        "{}.{}.{}.{}",
        octets[0], octets[1], octets[2], octets[3]
    )))
}

fn is_private_ipv4(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.is_private_ipv4")?;
    let ip: Ipv4Addr = addr
        .parse()
        .map_err(|_| ApexError::new("Expected valid IPv4 address"))?;
    let octets = ip.octets();
    let private = match octets {
        [10, ..] => true,
        [172, second, ..] if (16..=31).contains(&second) => true,
        [192, 168, ..] => true,
        [169, 254, ..] => true,
        _ => false,
    };
    Ok(Value::Bool(private))
}

fn cidr_contains(args: &[Value]) -> Result<Value, ApexError> {
    let cidr = expect_string_arg(args, 0, "net.cidr_contains")?;
    let target = expect_string_arg(args, 1, "net.cidr_contains")?;
    let (network_ip, prefix) = parse_cidr(&cidr)?;
    let mask = prefix_mask(prefix);
    let target_ip = parse_ipv4_literal(&target, "net.cidr_contains")?;
    Ok(Value::Bool(
        (ipv4_to_u32(network_ip) & mask) == (ipv4_to_u32(target_ip) & mask),
    ))
}

fn cidr_overlap(args: &[Value]) -> Result<Value, ApexError> {
    if args.len() != 2 {
        return Err(ApexError::new("net.cidr_overlap expects two CIDR strings"));
    }
    let first = expect_string_arg(args, 0, "net.cidr_overlap")?;
    let second = expect_string_arg(args, 1, "net.cidr_overlap")?;
    let (first_base, first_prefix) = parse_cidr(&first)?;
    let (second_base, second_prefix) = parse_cidr(&second)?;
    let (start_a, end_a) = cidr_bounds(first_base, first_prefix);
    let (start_b, end_b) = cidr_bounds(second_base, second_prefix);
    Ok(Value::Bool(start_a <= end_b && start_b <= end_a))
}

fn ipv4_network(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.ipv4_network")?;
    let prefix = expect_int_arg(args, 1, "net.ipv4_network")?;
    let prefix = prefix
        .to_u32()
        .ok_or_else(|| ApexError::new("net.ipv4_network expects prefix in [0, 32]"))?;
    if prefix > 32 {
        return Err(ApexError::new("net.ipv4_network expects prefix in [0, 32]"));
    }
    let ip = parse_ipv4_literal(&addr, "net.ipv4_network")?;
    let mask = prefix_mask(prefix);
    let network = ipv4_to_u32(ip) & mask;
    Ok(Value::String(format_ipv4(network)))
}

fn ipv4_broadcast(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.ipv4_broadcast")?;
    let prefix = expect_int_arg(args, 1, "net.ipv4_broadcast")?;
    let prefix = prefix
        .to_u32()
        .ok_or_else(|| ApexError::new("net.ipv4_broadcast expects prefix in [0, 32]"))?;
    if prefix > 32 {
        return Err(ApexError::new(
            "net.ipv4_broadcast expects prefix in [0, 32]",
        ));
    }
    let ip = parse_ipv4_literal(&addr, "net.ipv4_broadcast")?;
    let mask = prefix_mask(prefix);
    let broadcast = (ipv4_to_u32(ip) & mask) | !mask;
    Ok(Value::String(format_ipv4(broadcast)))
}

fn ipv4_range(args: &[Value]) -> Result<Value, ApexError> {
    let cidr = expect_string_arg(args, 0, "net.ipv4_range")?;
    let (network, prefix) = parse_cidr(&cidr)?;
    let mask = prefix_mask(prefix);
    let start = ipv4_to_u32(network) & mask;
    let end = start | (!mask);
    Ok(Value::Tuple(vec![
        Value::String(format_ipv4(start)),
        Value::String(format_ipv4(end)),
    ]))
}

fn ipv4_same_subnet(args: &[Value]) -> Result<Value, ApexError> {
    if args.len() != 3 {
        return Err(ApexError::new(
            "net.ipv4_same_subnet expects (addr_a, addr_b, prefix)",
        ));
    }
    let first = expect_string_arg(args, 0, "net.ipv4_same_subnet")?;
    let second = expect_string_arg(args, 1, "net.ipv4_same_subnet")?;
    let prefix = expect_int_arg(args, 2, "net.ipv4_same_subnet")?;
    let prefix = prefix
        .to_u32()
        .ok_or_else(|| ApexError::new("net.ipv4_same_subnet expects prefix in [0, 32]"))?;
    if prefix > 32 {
        return Err(ApexError::new(
            "net.ipv4_same_subnet expects prefix in [0, 32]",
        ));
    }
    let mask = prefix_mask(prefix);
    let left = ipv4_to_u32(parse_ipv4_literal(&first, "net.ipv4_same_subnet")?);
    let right = ipv4_to_u32(parse_ipv4_literal(&second, "net.ipv4_same_subnet")?);
    Ok(Value::Bool((left & mask) == (right & mask)))
}

fn ipv4_supernet(args: &[Value]) -> Result<Value, ApexError> {
    let cidr = expect_string_arg(args, 0, "net.ipv4_supernet")?;
    let (network, prefix) = parse_cidr(&cidr)?;
    if prefix == 0 {
        return Err(ApexError::new(
            "net.ipv4_supernet expects a prefix larger than 0",
        ));
    }
    let parent_prefix = prefix - 1;
    let parent_mask = prefix_mask(parent_prefix);
    let base = ipv4_to_u32(network) & parent_mask;
    Ok(Value::String(format!(
        "{}/{}",
        format_ipv4(base),
        parent_prefix
    )))
}

fn cidr_split(args: &[Value]) -> Result<Value, ApexError> {
    let cidr = expect_string_arg(args, 0, "net.cidr_split")?;
    let (network, prefix) = parse_cidr(&cidr)?;
    if prefix == 32 {
        return Err(ApexError::new(
            "net.cidr_split cannot subdivide a /32 network",
        ));
    }
    let child_prefix = prefix + 1;
    let base = ipv4_to_u32(network) & prefix_mask(prefix);
    let block = 1u32 << (32 - child_prefix);
    let first = base;
    let second = base + block;
    Ok(Value::Tuple(vec![
        Value::String(format!("{}/{}", format_ipv4(first), child_prefix)),
        Value::String(format!("{}/{}", format_ipv4(second), child_prefix)),
    ]))
}

fn ipv4_to_int(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.ipv4_to_int")?;
    let ip = parse_ipv4_literal(&addr, "net.ipv4_to_int")?;
    Ok(Value::Int(ipv4_to_u32(ip).into()))
}

fn int_to_ipv4(args: &[Value]) -> Result<Value, ApexError> {
    let raw = expect_int_arg(args, 0, "net.int_to_ipv4")?;
    let value = raw
        .to_u64()
        .ok_or_else(|| ApexError::new("net.int_to_ipv4 expects unsigned integer"))?;
    if value > u32::MAX as u64 {
        return Err(ApexError::new(
            "net.int_to_ipv4 expects a value that fits in 32 bits",
        ));
    }
    Ok(Value::String(format_ipv4(value as u32)))
}

fn ipv4_class(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.ipv4_class")?;
    let ip = parse_ipv4_literal(&addr, "net.ipv4_class")?;
    let octet = ip.octets()[0];
    let class = match octet {
        0..=127 => "A",
        128..=191 => "B",
        192..=223 => "C",
        224..=239 => "D",
        _ => "E",
    };
    Ok(Value::String(class.into()))
}

fn is_loopback(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.is_loopback")?;
    let ip = parse_ipv4_literal(&addr, "net.is_loopback")?;
    Ok(Value::Bool(ip.octets()[0] == 127))
}

fn ipv4_next(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.ipv4_next")?;
    let ip = parse_ipv4_literal(&addr, "net.ipv4_next")?;
    let numeric = ipv4_to_u32(ip);
    if numeric == u32::MAX {
        return Err(ApexError::new("Cannot increment the broadcast address"));
    }
    Ok(Value::String(format_ipv4(numeric + 1)))
}

fn ipv4_prev(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.ipv4_prev")?;
    let ip = parse_ipv4_literal(&addr, "net.ipv4_prev")?;
    let numeric = ipv4_to_u32(ip);
    if numeric == 0 {
        return Err(ApexError::new("Cannot decrement 0.0.0.0"));
    }
    Ok(Value::String(format_ipv4(numeric - 1)))
}

fn ipv4_host_count(args: &[Value]) -> Result<Value, ApexError> {
    let prefix = expect_int_arg(args, 0, "net.ipv4_host_count")?;
    let prefix = prefix
        .to_u32()
        .ok_or_else(|| ApexError::new("net.ipv4_host_count expects prefix in [0, 32]"))?;
    if prefix > 32 {
        return Err(ApexError::new(
            "net.ipv4_host_count expects prefix in [0, 32]",
        ));
    }
    let host_bits = 32 - prefix;
    let count = BigInt::from(1u8) << host_bits;
    Ok(Value::Int(count))
}

fn reverse_ptr(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.reverse_ptr")?;
    let ip = parse_ipv4_literal(&addr, "net.reverse_ptr")?;
    let octets = ip.octets();
    Ok(Value::String(format!(
        "{}.{}.{}.{}.in-addr.arpa",
        octets[3], octets[2], octets[1], octets[0]
    )))
}

fn is_multicast(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.is_multicast")?;
    let ip = parse_ipv4_literal(&addr, "net.is_multicast")?;
    let first = ip.octets()[0];
    Ok(Value::Bool((224..=239).contains(&first)))
}

fn mask_to_prefix(args: &[Value]) -> Result<Value, ApexError> {
    let mask = expect_string_arg(args, 0, "net.mask_to_prefix")?;
    let ip = parse_ipv4_literal(&mask, "net.mask_to_prefix")?;
    let mut bits = ipv4_to_u32(ip);
    let mut count = 0u32;
    let mut seen_zero = false;
    for _ in 0..32 {
        if bits & 0x8000_0000 != 0 {
            if seen_zero {
                return Err(ApexError::new(
                    "net.mask_to_prefix expects contiguous ones in the mask",
                ));
            }
            count += 1;
        } else {
            seen_zero = true;
        }
        bits <<= 1;
    }
    Ok(Value::Int(BigInt::from(count)))
}

fn ipv4_to_binary(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.ipv4_to_binary")?;
    let ip = parse_ipv4_literal(&addr, "net.ipv4_to_binary")?;
    let octets = ip.octets();
    Ok(Value::String(format!(
        "{:08b}.{:08b}.{:08b}.{:08b}",
        octets[0], octets[1], octets[2], octets[3]
    )))
}

fn is_link_local(args: &[Value]) -> Result<Value, ApexError> {
    let addr = expect_string_arg(args, 0, "net.is_link_local")?;
    let ip = parse_ipv4_literal(&addr, "net.is_link_local")?;
    let octets = ip.octets();
    Ok(Value::Bool(octets[0] == 169 && octets[1] == 254))
}

fn parse_cidr(text: &str) -> Result<(Ipv4Addr, u32), ApexError> {
    let mut parts = text.split('/');
    let base = parts
        .next()
        .ok_or_else(|| ApexError::new("CIDR expects address/prefix"))?;
    let prefix = parts
        .next()
        .ok_or_else(|| ApexError::new("CIDR expects address/prefix"))?;
    if parts.next().is_some() {
        return Err(ApexError::new("CIDR expects address/prefix"));
    }
    let prefix_value: u32 = prefix
        .parse()
        .map_err(|_| ApexError::new("CIDR prefix must be integer"))?;
    if prefix_value > 32 {
        return Err(ApexError::new("CIDR prefix must be between 0 and 32"));
    }
    let base_ip = parse_ipv4_literal(base, "net.cidr_contains")?;
    Ok((base_ip, prefix_value))
}

fn parse_ipv4_literal(input: &str, name: &str) -> Result<Ipv4Addr, ApexError> {
    input
        .parse()
        .map_err(|_| ApexError::new(format!("{} expects a valid IPv4 address", name)))
}

fn prefix_mask(bits: u32) -> u32 {
    if bits == 0 {
        0
    } else {
        u32::MAX << (32 - bits)
    }
}

fn ipv4_to_u32(addr: Ipv4Addr) -> u32 {
    u32::from(addr)
}

fn cidr_bounds(network: Ipv4Addr, prefix: u32) -> (u32, u32) {
    let mask = prefix_mask(prefix);
    let start = ipv4_to_u32(network) & mask;
    let end = start | (!mask);
    (start, end)
}

fn format_ipv4(value: u32) -> String {
    let octets = value.to_be_bytes();
    format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_localhost() {
        let result = resolve_host(&[Value::String("localhost".into())]).unwrap();
        if let Value::Tuple(values) = result {
            assert!(!values.is_empty());
        } else {
            panic!("expected tuple");
        }
        assert_eq!(
            parse_ipv4(&[Value::String("127.0.0.1".into())]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            subnet_mask(&[Value::Int(24.into())]).unwrap(),
            Value::String("255.255.255.0".into())
        );
        assert_eq!(
            is_private_ipv4(&[Value::String("192.168.0.1".into())]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            is_private_ipv4(&[Value::String("8.8.8.8".into())]).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            cidr_contains(&[
                Value::String("192.168.0.0/24".into()),
                Value::String("192.168.0.42".into()),
            ])
            .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ipv4_network(&[Value::String("10.0.1.5".into()), Value::Int(16.into())]).unwrap(),
            Value::String("10.0.0.0".into())
        );
        assert_eq!(
            ipv4_broadcast(&[Value::String("10.0.1.5".into()), Value::Int(24.into())]).unwrap(),
            Value::String("10.0.1.255".into())
        );
        let range = ipv4_range(&[Value::String("10.0.1.0/24".into())]).unwrap();
        if let Value::Tuple(values) = range {
            assert_eq!(values[0], Value::String("10.0.1.0".into()));
            assert_eq!(values[1], Value::String("10.0.1.255".into()));
        } else {
            panic!("expected tuple");
        }
        assert_eq!(
            ipv4_supernet(&[Value::String("10.0.1.0/24".into())]).unwrap(),
            Value::String("10.0.0.0/23".into())
        );
        assert_eq!(
            ipv4_same_subnet(&[
                Value::String("10.0.0.1".into()),
                Value::String("10.0.1.2".into()),
                Value::Int(23.into()),
            ])
            .unwrap(),
            Value::Bool(true)
        );
        let split = cidr_split(&[Value::String("10.0.0.0/23".into())]).unwrap();
        if let Value::Tuple(values) = split {
            assert_eq!(values[0], Value::String("10.0.0.0/24".into()));
            assert_eq!(values[1], Value::String("10.0.1.0/24".into()));
        } else {
            panic!("expected split tuple");
        }
        assert_eq!(
            cidr_overlap(&[
                Value::String("10.0.0.0/24".into()),
                Value::String("10.0.0.128/25".into()),
            ])
            .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ipv4_to_int(&[Value::String("127.0.0.1".into())]).unwrap(),
            Value::Int(2130706433u32.into())
        );
        assert_eq!(
            int_to_ipv4(&[Value::Int(3232235777u64.into())]).unwrap(),
            Value::String("192.168.1.1".into())
        );
        assert_eq!(
            ipv4_class(&[Value::String("224.0.0.1".into())]).unwrap(),
            Value::String("D".into())
        );
        assert_eq!(
            is_loopback(&[Value::String("127.1.2.3".into())]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ipv4_next(&[Value::String("192.168.0.1".into())]).unwrap(),
            Value::String("192.168.0.2".into())
        );
        assert_eq!(
            ipv4_prev(&[Value::String("192.168.0.2".into())]).unwrap(),
            Value::String("192.168.0.1".into())
        );
        assert_eq!(
            ipv4_host_count(&[Value::Int(24.into())]).unwrap(),
            Value::Int(256.into())
        );
        assert_eq!(
            reverse_ptr(&[Value::String("1.2.3.4".into())]).unwrap(),
            Value::String("4.3.2.1.in-addr.arpa".into())
        );
        assert_eq!(
            is_multicast(&[Value::String("239.1.2.3".into())]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            mask_to_prefix(&[Value::String("255.255.0.0".into())]).unwrap(),
            Value::Int(16.into())
        );
        let binary = ipv4_to_binary(&[Value::String("10.0.0.1".into())]).unwrap();
        if let Value::String(bits) = binary {
            assert!(bits.starts_with("00001010"));
        }
        assert_eq!(
            is_link_local(&[Value::String("169.254.1.1".into())]).unwrap(),
            Value::Bool(true)
        );
    }
}
