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
                NativeCallable::new(concat!("network::", $name), $func),
            );
        };
    }

    add!(&mut functions, "resolve_host", resolve_host);
    add!(&mut functions, "parse_ipv4", parse_ipv4);
    add!(&mut functions, "subnet_mask", subnet_mask);
    add!(&mut functions, "is_private_ipv4", is_private_ipv4);
    registry.register_module("network", functions);
}

fn resolve_host(args: &[Value]) -> Result<Value, ApexError> {
    let host = expect_string_arg(args, 0, "network.resolve_host")?;
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
    let addr = expect_string_arg(args, 0, "network.parse_ipv4")?;
    Ok(Value::Bool(addr.parse::<Ipv4Addr>().is_ok()))
}

fn subnet_mask(args: &[Value]) -> Result<Value, ApexError> {
    let bits = expect_int_arg(args, 0, "network.subnet_mask")?;
    let bits = bits.to_u32().ok_or_else(|| {
        ApexError::new("network.subnet_mask expects a prefix length between 0 and 32")
    })?;
    if bits > 32 {
        return Err(ApexError::new(
            "network.subnet_mask expects a prefix length between 0 and 32",
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
    let addr = expect_string_arg(args, 0, "network.is_private_ipv4")?;
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
    }
}
