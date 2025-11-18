use std::collections::{HashMap, HashSet};
use std::net::{Ipv4Addr, ToSocketAddrs};

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
    add!(&mut functions, "ipv4_network", ipv4_network);
    add!(&mut functions, "ipv4_broadcast", ipv4_broadcast);
    add!(&mut functions, "ipv4_to_int", ipv4_to_int);
    add!(&mut functions, "int_to_ipv4", int_to_ipv4);
    add!(&mut functions, "ipv4_class", ipv4_class);
    add!(&mut functions, "is_loopback", is_loopback);
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
    }
}
