use std::collections::{HashMap, HashSet};

use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

use super::{NativeCallable, NativeRegistry};

pub(super) fn register(registry: &mut NativeRegistry) {
    let mut functions: HashMap<String, NativeCallable> = HashMap::new();

    macro_rules! add {
        ($map:expr, $name:literal, $func:ident) => {
            $map.insert(
                $name.to_string(),
                NativeCallable::new(concat!("nats::", $name), $func),
            );
        };
    }

    add!(functions, "btoi", btoi);
    add!(functions, "abs_value", abs_value);
    add!(functions, "sum_digits", sum_digits);
    add!(functions, "sum_digits_base", sum_digits_base);
    add!(functions, "num_digits", num_digits);
    add!(functions, "divisors_count", divisors_count);
    add!(functions, "divisors_sum", divisors_sum);
    add!(functions, "proper_divisors_sum", proper_divisors_sum);
    add!(functions, "is_perfect", is_perfect);
    add!(functions, "is_abundant", is_abundant);
    add!(functions, "is_deficient", is_deficient);
    add!(functions, "is_prime", is_prime);
    add!(functions, "is_composite", is_composite);
    add!(functions, "is_simple_number", is_simple_number);
    add!(functions, "is_murekkeb_number", is_murekkeb_number);
    add!(functions, "is_twin_prime", is_twin_prime);
    add!(
        functions,
        "is_sophie_germain_prime",
        is_sophie_germain_prime
    );
    add!(functions, "is_cunningham_prime", is_cunningham_prime);
    add!(functions, "fermat_little", fermat_little);
    add!(functions, "is_fermat_pseudoprime", is_fermat_pseudoprime);
    add!(functions, "is_strong_pseudoprime", is_strong_pseudoprime);
    add!(functions, "miller_rabin_test", miller_rabin_test);
    add!(functions, "is_harshad", is_harshad);
    add!(functions, "is_armstrong", is_armstrong);
    add!(functions, "gcd", gcd);
    add!(functions, "lcm", lcm);
    add!(functions, "coprime", coprime);
    add!(functions, "is_even", is_even);
    add!(functions, "is_odd", is_odd);
    add!(functions, "next_even", next_even);
    add!(functions, "next_odd", next_odd);
    add!(functions, "fib", fib);
    add!(functions, "kaprekar_constant", kaprekar_constant);
    add!(functions, "is_kaprekar", is_kaprekar);
    add!(functions, "kaprekar_theorem", kaprekar_theorem);
    add!(functions, "kaprekar_6174_steps", kaprekar_6174_steps);
    add!(functions, "wilson_theorem", wilson_theorem);
    add!(functions, "phi", phi);
    add!(functions, "digital_root", digital_root);
    add!(functions, "fact", fact);
    add!(functions, "nCr", ncr);
    add!(functions, "modpow", modpow);
    add!(functions, "modinv", modinv);
    add!(functions, "prime_count_up_to", prime_count_up_to);
    add!(functions, "is_amicable", is_amicable);
    add!(functions, "aliquot_length", aliquot_length);
    add!(functions, "goldbach_holds", goldbach_holds);
    add!(functions, "goldbach_witness", goldbach_witness);
    add!(functions, "is_square", is_square);
    add!(functions, "is_power", is_power);
    add!(functions, "mobius", mobius);
    add!(functions, "legendre_symbol", legendre_symbol);
    add!(functions, "is_quadratic_residue", is_quadratic_residue);
    add!(functions, "carmichael", carmichael);
    add!(functions, "is_carmichael", is_carmichael);
    add!(functions, "mersenne_number", mersenne_number);
    add!(functions, "is_mersenne_prime", is_mersenne_prime);
    add!(functions, "lucas_lehmer", lucas_lehmer);

    registry.register_module("nats", functions);
}

fn ensure_len(args: &[Value], expected: usize, name: &str) -> Result<(), ApexError> {
    if args.len() != expected {
        return Err(ApexError::new(format!(
            "{} expected {} argument(s) but received {}",
            name,
            expected,
            args.len()
        )));
    }
    Ok(())
}

fn expect_bool_arg(args: &[Value], index: usize, name: &str) -> Result<bool, ApexError> {
    match args.get(index) {
        Some(Value::Bool(v)) => Ok(*v),
        _ => Err(ApexError::new(format!(
            "{} expects boolean argument at position {}",
            name,
            index + 1
        ))),
    }
}

fn expect_int_arg(args: &[Value], index: usize, name: &str) -> Result<BigInt, ApexError> {
    match args.get(index) {
        Some(Value::Int(v)) => Ok(v.clone()),
        _ => Err(ApexError::new(format!(
            "{} expects integer argument at position {}",
            name,
            index + 1
        ))),
    }
}

fn expect_nat_arg(args: &[Value], index: usize, name: &str) -> Result<BigInt, ApexError> {
    let value = expect_int_arg(args, index, name)?;
    if value.sign() == Sign::Minus {
        return Err(ApexError::new(format!(
            "{} expects non-negative integer at position {}",
            name,
            index + 1
        )));
    }
    Ok(value)
}

fn to_usize(value: &BigInt, name: &str) -> Result<usize, ApexError> {
    value.to_usize().ok_or_else(|| {
        ApexError::new(format!(
            "{} argument '{}' is too large to fit in usize",
            name, value
        ))
    })
}

fn to_u32(value: &BigInt, name: &str) -> Result<u32, ApexError> {
    value.to_u32().ok_or_else(|| {
        ApexError::new(format!(
            "{} argument '{}' is too large to fit in u32",
            name, value
        ))
    })
}

fn to_u128(value: &BigInt, name: &str) -> Result<u128, ApexError> {
    value.to_u128().ok_or_else(|| {
        ApexError::new(format!(
            "{} argument '{}' is too large to fit in u128",
            name, value
        ))
    })
}

fn ensure_base_range(base: &BigInt, n: &BigInt, name: &str) -> Result<(), ApexError> {
    let one = BigInt::one();
    if base <= &one || base >= n {
        return Err(ApexError::new(format!(
            "{} requires base satisfying 1 < base < n",
            name
        )));
    }
    Ok(())
}

fn big_one() -> BigInt {
    BigInt::one()
}

fn big_zero() -> BigInt {
    BigInt::zero()
}

fn btoi(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "btoi")?;
    Ok(Value::Int(if expect_bool_arg(args, 0, "btoi")? {
        big_one()
    } else {
        big_zero()
    }))
}

fn abs_value(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "abs_value")?;
    let value = expect_int_arg(args, 0, "abs_value")?;
    Ok(Value::Int(value.abs()))
}

fn sum_digits(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "sum_digits")?;
    let value = expect_nat_arg(args, 0, "sum_digits")?;
    Ok(Value::Int(sum_digits_impl(&value, 10)))
}

fn sum_digits_base(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "sum_digits_base")?;
    let value = expect_nat_arg(args, 0, "sum_digits_base")?;
    let base = expect_nat_arg(args, 1, "sum_digits_base")?;
    let radix = to_u128(&base, "sum_digits_base")?;
    if radix < 2 {
        return Err(ApexError::new("sum_digits_base requires base >= 2"));
    }
    Ok(Value::Int(sum_digits_impl(&value, radix as u32)))
}

fn num_digits(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "num_digits")?;
    let value = expect_nat_arg(args, 0, "num_digits")?;
    if value.is_zero() {
        return Ok(Value::Int(big_zero()));
    }
    Ok(Value::Int(
        BigInt::from(value.to_str_radix(10).len() as u64),
    ))
}

fn sum_digits_impl(value: &BigInt, base: u32) -> BigInt {
    if value.is_zero() {
        return big_zero();
    }
    let mut n = value.abs();
    let mut sum = big_zero();
    while !n.is_zero() {
        let (q, r) = n.div_rem(&BigInt::from(base));
        sum += r;
        n = q;
    }
    sum
}

fn divisors_count(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "divisors_count")?;
    let n = expect_nat_arg(args, 0, "divisors_count")?;
    let factors = prime_factors_u128(&n, "divisors_count")?;
    let mut count = 1u128;
    for (_, exp) in factors {
        count *= exp as u128 + 1;
    }
    Ok(Value::Int(BigInt::from(count)))
}

fn divisors_sum(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "divisors_sum")?;
    let n = expect_nat_arg(args, 0, "divisors_sum")?;
    let sum = sum_of_divisors(&n, "divisors_sum")?;
    Ok(Value::Int(sum))
}

fn proper_divisors_sum(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "proper_divisors_sum")?;
    let n = expect_nat_arg(args, 0, "proper_divisors_sum")?;
    if n.is_zero() {
        return Err(ApexError::new("proper_divisors_sum is undefined for zero"));
    }
    let sum = sum_of_divisors(&n, "proper_divisors_sum")? - &n;
    Ok(Value::Int(sum))
}

fn is_perfect(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_perfect")?;
    let n = expect_nat_arg(args, 0, "is_perfect")?;
    if n.is_zero() {
        return Ok(Value::Bool(false));
    }
    let sum = sum_of_divisors(&n, "is_perfect")? - &n;
    Ok(Value::Bool(sum == n))
}

fn is_abundant(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_abundant")?;
    let n = expect_nat_arg(args, 0, "is_abundant")?;
    if n.is_zero() {
        return Ok(Value::Bool(false));
    }
    let sum = sum_of_divisors(&n, "is_abundant")? - &n;
    Ok(Value::Bool(sum > n))
}

fn is_deficient(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_deficient")?;
    let n = expect_nat_arg(args, 0, "is_deficient")?;
    if n.is_zero() {
        return Ok(Value::Bool(false));
    }
    let sum = sum_of_divisors(&n, "is_deficient")? - &n;
    Ok(Value::Bool(sum < n))
}

fn is_prime(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_prime")?;
    let n = expect_nat_arg(args, 0, "is_prime")?;
    Ok(Value::Bool(is_prime_u128(&n, "is_prime")?))
}

fn is_composite(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_composite")?;
    let n = expect_nat_arg(args, 0, "is_composite")?;
    let is_prime = is_prime_u128(&n, "is_composite")?;
    Ok(Value::Bool(n > BigInt::one() && !is_prime))
}

fn is_simple_number(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_simple_number")?;
    let n = expect_nat_arg(args, 0, "is_simple_number")?;
    Ok(Value::Bool(is_prime_u128(&n, "is_simple_number")?))
}

fn is_murekkeb_number(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_murekkeb_number")?;
    let n = expect_nat_arg(args, 0, "is_murekkeb_number")?;
    let is_prime = is_prime_u128(&n, "is_murekkeb_number")?;
    Ok(Value::Bool(n > BigInt::one() && !is_prime))
}

fn is_twin_prime(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_twin_prime")?;
    let n = expect_nat_arg(args, 0, "is_twin_prime")?;
    if !is_prime_u128(&n, "is_twin_prime")? {
        return Ok(Value::Bool(false));
    }
    let two = BigInt::from(2u8);
    let mut has_partner = false;
    if n > two {
        let lower = &n - &two;
        has_partner |= is_prime_u128(&lower, "is_twin_prime")?;
    }
    if !has_partner {
        let upper = &n + &two;
        has_partner = is_prime_u128(&upper, "is_twin_prime")?;
    }
    Ok(Value::Bool(has_partner))
}

fn is_sophie_germain_prime(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_sophie_germain_prime")?;
    let n = expect_nat_arg(args, 0, "is_sophie_germain_prime")?;
    if !is_prime_u128(&n, "is_sophie_germain_prime")? {
        return Ok(Value::Bool(false));
    }
    let doubled = &n * BigInt::from(2u8) + BigInt::one();
    Ok(Value::Bool(is_prime_u128(
        &doubled,
        "is_sophie_germain_prime",
    )?))
}

fn is_cunningham_prime(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_cunningham_prime")?;
    let n = expect_nat_arg(args, 0, "is_cunningham_prime")?;
    if n <= BigInt::one() || !is_prime_u128(&n, "is_cunningham_prime")? {
        return Ok(Value::Bool(false));
    }
    let partner = &n * BigInt::from(2u8) - BigInt::one();
    if partner <= BigInt::one() {
        return Ok(Value::Bool(false));
    }
    Ok(Value::Bool(is_prime_u128(&partner, "is_cunningham_prime")?))
}

fn fermat_little(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "fermat_little")?;
    let base = expect_nat_arg(args, 0, "fermat_little")?;
    let modulus = expect_nat_arg(args, 1, "fermat_little")?;
    if modulus <= BigInt::one() {
        return Err(ApexError::new("fermat_little requires modulus > 1"));
    }
    ensure_base_range(&base, &modulus, "fermat_little")?;
    if !base.gcd(&modulus).is_one() {
        return Ok(Value::Bool(false));
    }
    let exponent = &modulus - BigInt::one();
    let residue = mod_pow(base, exponent, modulus.clone());
    Ok(Value::Bool(residue == BigInt::one()))
}

fn is_fermat_pseudoprime(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "is_fermat_pseudoprime")?;
    let n = expect_nat_arg(args, 0, "is_fermat_pseudoprime")?;
    let base = expect_nat_arg(args, 1, "is_fermat_pseudoprime")?;
    if n <= BigInt::from(3u8) {
        return Ok(Value::Bool(false));
    }
    ensure_base_range(&base, &n, "is_fermat_pseudoprime")?;
    if base.gcd(&n).is_one() {
        let exponent = &n - BigInt::one();
        Ok(Value::Bool(
            mod_pow(base, exponent, n.clone()) == BigInt::one(),
        ))
    } else {
        Ok(Value::Bool(false))
    }
}

fn is_strong_pseudoprime(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "is_strong_pseudoprime")?;
    let n = expect_nat_arg(args, 0, "is_strong_pseudoprime")?;
    let base = expect_nat_arg(args, 1, "is_strong_pseudoprime")?;
    if n <= BigInt::from(3u8) {
        return Ok(Value::Bool(false));
    }
    ensure_base_range(&base, &n, "is_strong_pseudoprime")?;
    Ok(Value::Bool(strong_probable_prime(&n, &base)))
}

fn miller_rabin_test(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "miller_rabin_test")?;
    let n = expect_nat_arg(args, 0, "miller_rabin_test")?;
    let rounds_value = expect_nat_arg(args, 1, "miller_rabin_test")?;
    let rounds = to_usize(&rounds_value, "miller_rabin_test")?;
    if rounds == 0 {
        return Err(ApexError::new(
            "miller_rabin_test requires at least one round",
        ));
    }
    if n <= BigInt::from(3u8) {
        return Ok(Value::Bool(
            n == BigInt::from(2u8) || n == BigInt::from(3u8),
        ));
    }
    if n.is_even() {
        return Ok(Value::Bool(false));
    }
    let n_u128 = to_u128(&n, "miller_rabin_test")?;
    let deterministic_bases: [u128; 7] = [2, 3, 5, 7, 11, 13, 17];
    let mut tests = 0usize;
    for &candidate in &deterministic_bases {
        if tests >= rounds {
            break;
        }
        if candidate >= n_u128 {
            continue;
        }
        if !strong_probable_prime(&n, &BigInt::from(candidate)) {
            return Ok(Value::Bool(false));
        }
        tests += 1;
    }
    let mut candidate = 19u128;
    while tests < rounds {
        let range = n_u128.saturating_sub(3);
        if range == 0 {
            break;
        }
        let base_value = (candidate % range) + 2;
        if !strong_probable_prime(&n, &BigInt::from(base_value)) {
            return Ok(Value::Bool(false));
        }
        tests += 1;
        candidate += 2;
    }
    Ok(Value::Bool(true))
}

fn is_harshad(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_harshad")?;
    let n = expect_nat_arg(args, 0, "is_harshad")?;
    if n.is_zero() {
        return Ok(Value::Bool(true));
    }
    let sum = sum_digits_impl(&n, 10);
    Ok(Value::Bool(
        !sum.is_zero() && (&n % sum.clone()) == big_zero(),
    ))
}

fn is_armstrong(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_armstrong")?;
    let n = expect_nat_arg(args, 0, "is_armstrong")?;
    let digits: Vec<u32> = n
        .to_str_radix(10)
        .chars()
        .map(|c| c.to_digit(10).unwrap())
        .collect();
    let power = digits.len() as u32;
    let mut sum = big_zero();
    for d in digits {
        sum += BigInt::from(d).pow(power);
    }
    Ok(Value::Bool(sum == n))
}

fn gcd(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "gcd")?;
    let a = expect_int_arg(args, 0, "gcd")?;
    let b = expect_int_arg(args, 1, "gcd")?;
    Ok(Value::Int(a.gcd(&b)))
}

fn lcm(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "lcm")?;
    let a = expect_int_arg(args, 0, "lcm")?;
    let b = expect_int_arg(args, 1, "lcm")?;
    if a.is_zero() || b.is_zero() {
        return Ok(Value::Int(big_zero()));
    }
    let gcd = a.gcd(&b);
    Ok(Value::Int((a / &gcd) * b))
}

fn coprime(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "coprime")?;
    let a = expect_int_arg(args, 0, "coprime")?;
    let b = expect_int_arg(args, 1, "coprime")?;
    Ok(Value::Bool(a.gcd(&b).is_one()))
}

fn is_even(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_even")?;
    let n = expect_int_arg(args, 0, "is_even")?;
    Ok(Value::Bool((&n & BigInt::one()).is_zero()))
}

fn is_odd(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_odd")?;
    let n = expect_int_arg(args, 0, "is_odd")?;
    Ok(Value::Bool(!(&n & BigInt::one()).is_zero()))
}

fn next_even(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "next_even")?;
    let mut n = expect_int_arg(args, 0, "next_even")?;
    if (&n & BigInt::one()).is_zero() {
        n += BigInt::from(2);
    } else {
        n += BigInt::one();
    }
    Ok(Value::Int(n))
}

fn next_odd(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "next_odd")?;
    let mut n = expect_int_arg(args, 0, "next_odd")?;
    if (&n & BigInt::one()).is_zero() {
        n += BigInt::one();
    } else {
        n += BigInt::from(2);
    }
    Ok(Value::Int(n))
}

fn fib(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "fib")?;
    let n = expect_nat_arg(args, 0, "fib")?;
    let count = to_usize(&n, "fib")?;
    let mut a = big_zero();
    let mut b = big_one();
    for _ in 0..count {
        let next = &a + &b;
        a = b;
        b = next;
    }
    Ok(Value::Int(a))
}

fn kaprekar_constant(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 0, "kaprekar_constant")?;
    Ok(Value::Int(BigInt::from(6174u32)))
}

fn is_kaprekar(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_kaprekar")?;
    let n = expect_nat_arg(args, 0, "is_kaprekar")?;
    if n.is_zero() || n == BigInt::one() {
        return Ok(Value::Bool(true));
    }
    let square = &n * &n;
    let ten = BigInt::from(10u8);
    let mut power = BigInt::one();
    while power <= square {
        let right = &square % &power;
        let left = &square / &power;
        if !right.is_zero() && left + right == n {
            return Ok(Value::Bool(true));
        }
        power *= &ten;
    }
    Ok(Value::Bool(false))
}

fn kaprekar_theorem(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "kaprekar_theorem")?;
    let n = expect_nat_arg(args, 0, "kaprekar_theorem")?;
    let value = n
        .to_u32()
        .ok_or_else(|| ApexError::new("kaprekar_theorem currently supports inputs up to 32-bit"))?;
    if value > 9999 {
        return Err(ApexError::new(
            "kaprekar_theorem expects a four-digit value (leading zeros allowed)",
        ));
    }
    let holds = kaprekar_reaches_constant(value);
    Ok(Value::Bool(holds))
}

fn kaprekar_6174_steps(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "kaprekar_6174_steps")?;
    let n = expect_nat_arg(args, 0, "kaprekar_6174_steps")?;
    let mut current = n.to_u32().ok_or_else(|| {
        ApexError::new("kaprekar_6174_steps currently supports inputs up to 32-bit")
    })?;
    let mut steps = 0u32;
    while current != 6174 && current != 0 {
        current = kaprekar_step(current);
        steps += 1;
        if steps > 100 {
            break;
        }
    }
    Ok(Value::Int(BigInt::from(steps)))
}

fn kaprekar_reaches_constant(mut value: u32) -> bool {
    if !kaprekar_has_distinct_digits(value) {
        return false;
    }
    for _ in 0..8 {
        if value == 6174 {
            return true;
        }
        value = kaprekar_step(value);
    }
    value == 6174
}

fn kaprekar_step(value: u32) -> u32 {
    let mut digits: Vec<u32> = format!("{value:04}")
        .chars()
        .map(|c| c.to_digit(10).unwrap())
        .collect();
    digits.sort();
    let small: u32 = digits.iter().fold(0, |acc, &d| acc * 10 + d);
    digits.reverse();
    let large: u32 = digits.iter().fold(0, |acc, &d| acc * 10 + d);
    large - small
}

fn kaprekar_has_distinct_digits(value: u32) -> bool {
    let digits: Vec<char> = format!("{value:04}").chars().collect();
    digits.iter().any(|d| *d != digits[0])
}

fn wilson_theorem(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "wilson_theorem")?;
    let n = expect_nat_arg(args, 0, "wilson_theorem")?;
    if n <= BigInt::one() {
        return Ok(Value::Bool(false));
    }
    let mut residue = BigInt::one();
    let mut candidate = BigInt::from(2u8);
    while candidate < n {
        residue = (residue * &candidate) % &n;
        candidate += BigInt::one();
    }
    let holds = (residue + BigInt::one()) % &n == BigInt::zero();
    Ok(Value::Bool(holds))
}

fn phi(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "phi")?;
    let n = expect_nat_arg(args, 0, "phi")?;
    if n.is_zero() {
        return Ok(Value::Int(big_zero()));
    }
    let mut result = n.clone();
    let factors = prime_factors_u128(&n, "phi")?;
    for (p, _) in factors {
        result -= &result / BigInt::from(p as u128);
    }
    Ok(Value::Int(result))
}

fn digital_root(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "digital_root")?;
    let mut n = expect_nat_arg(args, 0, "digital_root")?;
    while n >= BigInt::from(10u8) {
        n = sum_digits_impl(&n, 10);
    }
    Ok(Value::Int(n))
}

fn fact(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "fact")?;
    let n = expect_nat_arg(args, 0, "fact")?;
    let count = to_usize(&n, "fact")?;
    let mut result = big_one();
    for i in 2..=count {
        result *= BigInt::from(i as u64);
    }
    Ok(Value::Int(result))
}

fn ncr(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "nCr")?;
    let n = expect_nat_arg(args, 0, "nCr")?;
    let r = expect_nat_arg(args, 1, "nCr")?;
    if r > n {
        return Ok(Value::Int(big_zero()));
    }
    let n_usize = to_usize(&n, "nCr")?;
    let r_usize = to_usize(&r, "nCr")?;
    Ok(Value::Int(binomial(n_usize, r_usize)))
}

fn modpow(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 3, "modpow")?;
    let base = expect_nat_arg(args, 0, "modpow")?;
    let exp = expect_nat_arg(args, 1, "modpow")?;
    let modulus = expect_nat_arg(args, 2, "modpow")?;
    if modulus.is_zero() {
        return Err(ApexError::new("modpow requires modulus > 0"));
    }
    Ok(Value::Int(mod_pow(base, exp, modulus)))
}

fn modinv(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "modinv")?;
    let a = expect_nat_arg(args, 0, "modinv")?;
    let m = expect_nat_arg(args, 1, "modinv")?;
    if m.is_zero() {
        return Err(ApexError::new("modinv requires modulus > 0"));
    }
    let (g, x, _) = extended_gcd(a.clone(), m.clone());
    if !g.is_one() {
        return Err(ApexError::new("modinv requires coprime inputs"));
    }
    let mut result = x % &m;
    if result.sign() == Sign::Minus {
        result += m;
    }
    Ok(Value::Int(result))
}

fn prime_count_up_to(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "prime_count_up_to")?;
    let n = expect_nat_arg(args, 0, "prime_count_up_to")?;
    let limit = to_usize(&n, "prime_count_up_to")?;
    Ok(Value::Int(BigInt::from(sieve_prime_count(limit))))
}

fn is_amicable(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_amicable")?;
    let n = expect_nat_arg(args, 0, "is_amicable")?;
    if n.is_zero() {
        return Ok(Value::Bool(false));
    }
    let s1 = sum_of_divisors(&n, "is_amicable")? - &n;
    if s1.is_zero() {
        return Ok(Value::Bool(false));
    }
    let s2 = sum_of_divisors(&s1, "is_amicable")? - s1.clone();
    Ok(Value::Bool(s2 == n && s1 != n))
}

fn aliquot_length(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "aliquot_length")?;
    let mut n = expect_nat_arg(args, 0, "aliquot_length")?;
    let mut length = 0u32;
    let mut seen = HashSet::new();
    while n > big_zero() && length < 1000 {
        if !seen.insert(n.clone()) {
            break;
        }
        n = sum_of_divisors(&n, "aliquot_length")? - &n;
        length += 1;
    }
    Ok(Value::Int(BigInt::from(length)))
}

fn goldbach_holds(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "goldbach_holds")?;
    let n = expect_nat_arg(args, 0, "goldbach_holds")?;
    let target = validate_goldbach_target(&n, "goldbach_holds")?;
    Ok(Value::Bool(find_goldbach_pair(target).is_some()))
}

fn goldbach_witness(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "goldbach_witness")?;
    let n = expect_nat_arg(args, 0, "goldbach_witness")?;
    let target = validate_goldbach_target(&n, "goldbach_witness")?;
    if let Some((prime, _)) = find_goldbach_pair(target) {
        Ok(Value::Int(BigInt::from(prime)))
    } else {
        Err(ApexError::new("Goldbach search failed to find a witness"))
    }
}

fn is_square(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_square")?;
    let n = expect_nat_arg(args, 0, "is_square")?;
    let sqrt = integer_sqrt(&n);
    Ok(Value::Bool(&sqrt * &sqrt == n))
}

fn is_power(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "is_power")?;
    let n = expect_nat_arg(args, 0, "is_power")?;
    let k = expect_nat_arg(args, 1, "is_power")?;
    let exponent = to_usize(&k, "is_power")?;
    if exponent < 2 {
        return Err(ApexError::new("is_power exponent must be >= 2"));
    }
    if n.is_zero() {
        return Ok(Value::Bool(true));
    }
    let one = BigInt::one();
    let two = BigInt::from(2u8);
    let mut low = one.clone();
    let mut high = n.clone();
    while low <= high {
        let mid = (&low + &high) / &two;
        let power = mid.pow(exponent as u32);
        if power == n {
            return Ok(Value::Bool(true));
        }
        if power < n {
            low = mid + &one;
        } else {
            if mid == one {
                break;
            }
            high = mid - &one;
        }
    }
    Ok(Value::Bool(false))
}

fn mobius(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "mobius")?;
    let n = expect_nat_arg(args, 0, "mobius")?;
    if n.is_zero() {
        return Ok(Value::Int(big_zero()));
    }
    let factors = prime_factors_u128(&n, "mobius")?;
    for (_, exp) in &factors {
        if *exp > 1 {
            return Ok(Value::Int(big_zero()));
        }
    }
    let sign = if factors.len() % 2 == 0 { 1 } else { -1 };
    Ok(Value::Int(BigInt::from(sign)))
}

fn legendre_symbol(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "legendre_symbol")?;
    let a = expect_nat_arg(args, 0, "legendre_symbol")?;
    let p = expect_nat_arg(args, 1, "legendre_symbol")?;
    if p <= BigInt::from(2u8) {
        return Err(ApexError::new("legendre_symbol requires odd prime modulus"));
    }
    if !is_prime_u128(&p, "legendre_symbol")? {
        return Err(ApexError::new("legendre_symbol requires prime modulus"));
    }
    let value = mod_pow(
        a.clone(),
        (&p - BigInt::one()) / BigInt::from(2u8),
        p.clone(),
    );
    if value.is_zero() {
        Ok(Value::Int(big_zero()))
    } else if value == BigInt::one() {
        Ok(Value::Int(big_one()))
    } else {
        Ok(Value::Int(BigInt::from(-1)))
    }
}

fn is_quadratic_residue(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 2, "is_quadratic_residue")?;
    let a = expect_nat_arg(args, 0, "is_quadratic_residue")?;
    let p = expect_nat_arg(args, 1, "is_quadratic_residue")?;
    let legendre = legendre_symbol(&[Value::Int(a), Value::Int(p.clone())])?;
    if let Value::Int(v) = legendre {
        Ok(Value::Bool(v == BigInt::one()))
    } else {
        unreachable!()
    }
}

fn carmichael(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "carmichael")?;
    let n = expect_nat_arg(args, 0, "carmichael")?;
    if n <= BigInt::from(1u8) {
        return Ok(Value::Int(big_zero()));
    }
    let mut result = BigInt::one();
    let factors = prime_factors_u128(&n, "carmichael")?;
    for (prime, exponent) in factors {
        let component = carmichael_prime_power(prime, exponent);
        result = lcm_bigints(result, component);
    }
    Ok(Value::Int(result))
}

fn is_carmichael(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_carmichael")?;
    let n = expect_nat_arg(args, 0, "is_carmichael")?;
    if n <= BigInt::from(2u8) {
        return Ok(Value::Bool(false));
    }
    if is_prime_u128(&n, "is_carmichael")? {
        return Ok(Value::Bool(false));
    }
    let factors = prime_factors_u128(&n, "is_carmichael")?;
    if factors.is_empty() {
        return Ok(Value::Bool(false));
    }
    let n_minus_one = &n - BigInt::one();
    for (prime, exponent) in factors {
        if exponent > 1 {
            return Ok(Value::Bool(false));
        }
        let prime_minus_one = BigInt::from(prime - 1);
        if (&n_minus_one % prime_minus_one) != BigInt::zero() {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}

fn mersenne_number(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "mersenne_number")?;
    let exponent = expect_nat_arg(args, 0, "mersenne_number")?;
    let bits = to_u32(&exponent, "mersenne_number")?;
    if bits < 1 {
        return Err(ApexError::new("mersenne_number requires exponent >= 1"));
    }
    let value = (BigInt::one() << bits) - BigInt::one();
    Ok(Value::Int(value))
}

fn lucas_lehmer(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "lucas_lehmer")?;
    let exponent = expect_nat_arg(args, 0, "lucas_lehmer")?;
    let p = to_u32(&exponent, "lucas_lehmer")?;
    if p < 2 {
        return Err(ApexError::new("lucas_lehmer requires exponent >= 2"));
    }
    Ok(Value::Bool(lucas_lehmer_sequence(p)))
}

fn is_mersenne_prime(args: &[Value]) -> Result<Value, ApexError> {
    ensure_len(args, 1, "is_mersenne_prime")?;
    let exponent = expect_nat_arg(args, 0, "is_mersenne_prime")?;
    if !is_prime_u128(&exponent, "is_mersenne_prime")? {
        return Ok(Value::Bool(false));
    }
    let p = to_u32(&exponent, "is_mersenne_prime")?;
    if p < 2 {
        return Ok(Value::Bool(false));
    }
    Ok(Value::Bool(lucas_lehmer_sequence(p)))
}

fn sum_of_divisors(n: &BigInt, name: &str) -> Result<BigInt, ApexError> {
    if n.is_zero() {
        return Ok(big_zero());
    }
    let factors = prime_factors_u128(n, name)?;
    let mut result = BigInt::one();
    for (prime, exponent) in factors {
        let p = BigInt::from(prime as u128);
        let mut term = big_zero();
        let mut current = BigInt::one();
        for _ in 0..=exponent {
            term += &current;
            current *= &p;
        }
        result *= term;
    }
    Ok(result)
}

fn binomial(n: usize, r: usize) -> BigInt {
    let r = r.min(n - r);
    let mut numerator = BigInt::one();
    let mut denominator = BigInt::one();
    for i in 0..r {
        numerator *= BigInt::from((n - i) as u64);
        denominator *= BigInt::from((i + 1) as u64);
    }
    numerator / denominator
}

fn mod_pow(mut base: BigInt, mut exp: BigInt, modulus: BigInt) -> BigInt {
    let mut result = BigInt::one();
    base %= &modulus;
    while !exp.is_zero() {
        if (&exp & BigInt::one()).is_one() {
            result = (result * &base) % &modulus;
        }
        exp >>= 1;
        base = (&base * &base) % &modulus;
    }
    result
}

fn strong_probable_prime(n: &BigInt, base: &BigInt) -> bool {
    let one = BigInt::one();
    let two = BigInt::from(2u8);
    let three = BigInt::from(3u8);
    if n <= &three {
        return *n == two || *n == three;
    }
    if base <= &one || base >= n {
        return false;
    }
    if n.is_even() {
        return false;
    }
    if !base.gcd(n).is_one() {
        return false;
    }
    let mut d = n.clone() - &one;
    let mut s = 0u32;
    while d.is_even() {
        d >>= 1;
        s += 1;
    }
    let n_minus_one = n - &one;
    let mut x = mod_pow(base.clone(), d.clone(), n.clone());
    if x == one || x == n_minus_one {
        return true;
    }
    for _ in 1..s {
        x = (&x * &x) % n;
        if x == n_minus_one {
            return true;
        }
        if x == one {
            return false;
        }
    }
    false
}

fn extended_gcd(a: BigInt, b: BigInt) -> (BigInt, BigInt, BigInt) {
    if b.is_zero() {
        (a, BigInt::one(), big_zero())
    } else {
        let (g, x1, y1) = extended_gcd(b.clone(), a.clone() % b.clone());
        let x = y1.clone();
        let y = x1 - (a / b) * y1;
        (g, x, y)
    }
}

fn sieve_prime_count(limit: usize) -> usize {
    if limit < 2 {
        return 0;
    }
    let mut sieve = vec![true; limit + 1];
    sieve[0] = false;
    sieve[1] = false;
    let mut count = 0;
    for i in 2..=limit {
        if sieve[i] {
            count += 1;
            let mut j = i * 2;
            while j <= limit {
                sieve[j] = false;
                j += i;
            }
        }
    }
    count
}

fn validate_goldbach_target(n: &BigInt, name: &str) -> Result<u128, ApexError> {
    let target = to_u128(n, name)?;
    if target <= 4 || target % 2 != 0 {
        return Err(ApexError::new(format!(
            "{} requires an even integer greater than 4",
            name
        )));
    }
    Ok(target)
}

fn find_goldbach_pair(target: u128) -> Option<(u128, u128)> {
    let mut candidate = 2u128;
    while candidate <= target / 2 {
        if is_prime_raw(candidate) {
            let other = target - candidate;
            if is_prime_raw(other) {
                return Some((candidate, other));
            }
        }
        candidate += if candidate == 2 { 1 } else { 2 };
    }
    None
}

fn is_prime_raw(value: u128) -> bool {
    if value < 2 {
        return false;
    }
    if value == 2 || value == 3 {
        return true;
    }
    if value % 2 == 0 {
        return false;
    }
    let mut divisor = 3u128;
    while divisor * divisor <= value {
        if value % divisor == 0 {
            return false;
        }
        divisor += 2;
    }
    true
}

fn lucas_lehmer_sequence(p: u32) -> bool {
    if p == 2 {
        return true;
    }
    let mut s = BigInt::from(4u8);
    let modulus = (BigInt::one() << p) - BigInt::one();
    for _ in 0..(p - 2) {
        s = (&s * &s - BigInt::from(2u8)) % &modulus;
        if s.sign() == Sign::Minus {
            s += &modulus;
        }
    }
    s.is_zero()
}

fn prime_factors_u128(n: &BigInt, name: &str) -> Result<Vec<(u128, u32)>, ApexError> {
    if n.is_zero() {
        return Err(ApexError::new(format!("{} is undefined for zero", name)));
    }
    let mut remaining = to_u128(n, name)?;
    let mut factors = Vec::new();
    let mut divisor = 2u128;
    while divisor * divisor <= remaining {
        if remaining % divisor == 0 {
            let mut exp = 0u32;
            while remaining % divisor == 0 {
                remaining /= divisor;
                exp += 1;
            }
            factors.push((divisor, exp));
        }
        divisor += if divisor == 2 { 1 } else { 2 };
    }
    if remaining > 1 {
        factors.push((remaining, 1));
    }
    Ok(factors)
}

fn is_prime_u128(n: &BigInt, name: &str) -> Result<bool, ApexError> {
    let value = to_u128(n, name)?;
    Ok(is_prime_raw(value))
}

fn integer_sqrt(n: &BigInt) -> BigInt {
    if n.is_zero() {
        return big_zero();
    }
    let mut x = n.clone();
    let mut y = (&x + BigInt::one()) / BigInt::from(2u8);
    while y < x {
        x = y.clone();
        y = ((n / &x) + &x) / BigInt::from(2u8);
    }
    x
}

fn carmichael_prime_power(prime: u128, exponent: u32) -> BigInt {
    if prime == 2 {
        match exponent {
            1 => BigInt::one(),
            2 => BigInt::one(),
            _ => BigInt::from(1u128 << (exponent - 2)),
        }
    } else {
        BigInt::from((prime - 1) * prime.pow(exponent - 1))
    }
}

fn lcm_bigints(a: BigInt, b: BigInt) -> BigInt {
    if a.is_zero() || b.is_zero() {
        big_zero()
    } else {
        let g = a.gcd(&b);
        (a / g) * b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    fn int(n: i64) -> Value {
        Value::Int(BigInt::from(n))
    }

    fn uint(n: u64) -> Value {
        Value::Int(BigInt::from(n))
    }

    fn bool_value(b: bool) -> Value {
        Value::Bool(b)
    }

    #[test]
    fn sum_digits_handles_bases() {
        let ten = sum_digits(&[uint(98765)]).unwrap();
        assert_eq!(ten, uint(35));

        let hex = sum_digits_base(&[uint(0xFF), uint(16)]).unwrap();
        assert_eq!(hex, uint(30));
    }

    #[test]
    fn modpow_and_modinv_cover_edge_cases() {
        let value = modpow(&[uint(2), uint(10), uint(1000)]).unwrap();
        assert_eq!(value, uint(24));

        let inverse = modinv(&[uint(3), uint(11)]).unwrap();
        assert_eq!(inverse, uint(4));

        let modulus_err = modpow(&[uint(2), uint(5), uint(0)]);
        assert!(modulus_err.is_err());

        let coprime_err = modinv(&[uint(6), uint(15)]);
        assert!(coprime_err.is_err());
    }

    #[test]
    fn phi_and_mobius_behave_for_composites() {
        let phi_val = phi(&[uint(36)]).unwrap();
        assert_eq!(phi_val, uint(12));

        let mobius_squarefree = mobius(&[uint(30)]).unwrap();
        assert_eq!(mobius_squarefree, int(-1));

        let mobius_square = mobius(&[uint(12)]).unwrap();
        assert_eq!(mobius_square, uint(0));
    }

    #[test]
    fn legendre_and_quadratic_residue_classification() {
        let legendre_residue = legendre_symbol(&[uint(4), uint(7)]).unwrap();
        assert_eq!(legendre_residue, uint(1));

        let legendre_non_residue = legendre_symbol(&[uint(3), uint(7)]).unwrap();
        assert_eq!(legendre_non_residue, int(-1));

        let residue = is_quadratic_residue(&[uint(9), uint(11)]).unwrap();
        assert_eq!(residue, Value::Bool(true));

        let non_residue = is_quadratic_residue(&[uint(2), uint(11)]).unwrap();
        assert_eq!(non_residue, Value::Bool(false));

        let invalid = legendre_symbol(&[uint(1), uint(8)]);
        assert!(invalid.is_err());
    }

    #[test]
    fn twin_prime_aliases_and_wilson_checks() {
        let twin = is_twin_prime(&[uint(29)]).unwrap();
        assert_eq!(twin, bool_value(true));

        let non_twin = is_twin_prime(&[uint(27)]).unwrap();
        assert_eq!(non_twin, bool_value(false));

        let simple_alias = is_simple_number(&[uint(13)]).unwrap();
        assert_eq!(simple_alias, bool_value(true));

        let composite_alias = is_murekkeb_number(&[uint(45)]).unwrap();
        assert_eq!(composite_alias, bool_value(true));

        let wilson_prime = wilson_theorem(&[uint(13)]).unwrap();
        assert_eq!(wilson_prime, bool_value(true));

        let wilson_composite = wilson_theorem(&[uint(9)]).unwrap();
        assert_eq!(wilson_composite, bool_value(false));
    }

    #[test]
    fn kaprekar_utilities_cover_numbers_and_constant() {
        let constant = kaprekar_constant(&[]).unwrap();
        assert_eq!(constant, uint(6174));

        let kaprekar_number = is_kaprekar(&[uint(45)]).unwrap();
        assert_eq!(kaprekar_number, bool_value(true));

        let non_kaprekar = is_kaprekar(&[uint(10)]).unwrap();
        assert_eq!(non_kaprekar, bool_value(false));

        let kaprekar_steps = kaprekar_6174_steps(&[uint(3524)]).unwrap();
        assert_eq!(kaprekar_steps, uint(3));
    }

    #[test]
    fn prime_count_and_aliquot_length() {
        let primes = prime_count_up_to(&[uint(25)]).unwrap();
        assert_eq!(primes, uint(9));

        let length = aliquot_length(&[uint(12)]).unwrap();
        assert_eq!(length, uint(7));
    }

    #[test]
    fn powers_squares_and_carmichael() {
        assert_eq!(integer_sqrt(&BigInt::from(144)), BigInt::from(12));
        let cube = is_power(&[uint(27), uint(3)]).unwrap();
        assert_eq!(cube, Value::Bool(true));

        let non_cube = is_power(&[uint(20), uint(3)]).unwrap();
        assert_eq!(non_cube, Value::Bool(false));

        let bad_exponent = is_power(&[uint(8), uint(1)]);
        assert!(bad_exponent.is_err());

        let square_true = is_square(&[uint(144)]).unwrap();
        assert_eq!(square_true, Value::Bool(true));

        let square_false = is_square(&[uint(145)]).unwrap();
        assert_eq!(square_false, Value::Bool(false));

        let carmichael_val = carmichael(&[uint(45)]).unwrap();
        assert_eq!(carmichael_val, uint(12));
    }

    #[test]
    fn absolute_and_theorem_helpers() {
        let absolute = abs_value(&[int(-270)]).unwrap();
        assert_eq!(absolute, uint(270));

        let fermat_holds = fermat_little(&[uint(5), uint(97)]).unwrap();
        assert_eq!(fermat_holds, bool_value(true));

        let fermat_breaks = fermat_little(&[uint(5), uint(15)]).unwrap();
        assert_eq!(fermat_breaks, bool_value(false));

        let kaprekar_true = kaprekar_theorem(&[uint(3524)]).unwrap();
        assert_eq!(kaprekar_true, bool_value(true));

        let kaprekar_false = kaprekar_theorem(&[uint(1111)]).unwrap();
        assert_eq!(kaprekar_false, bool_value(false));

        let err = kaprekar_theorem(&[uint(20_000)]);
        assert!(err.is_err());
    }

    #[test]
    fn goldbach_and_prime_flavors() {
        let sophie = is_sophie_germain_prime(&[uint(23)]).unwrap();
        assert_eq!(sophie, bool_value(true));

        let sophie_false = is_sophie_germain_prime(&[uint(25)]).unwrap();
        assert_eq!(sophie_false, bool_value(false));

        let cunningham = is_cunningham_prime(&[uint(3)]).unwrap();
        assert_eq!(cunningham, bool_value(true));

        let cunningham_false = is_cunningham_prime(&[uint(11)]).unwrap();
        assert_eq!(cunningham_false, bool_value(false));

        let goldbach = goldbach_holds(&[uint(84)]).unwrap();
        assert_eq!(goldbach, bool_value(true));

        let witness = goldbach_witness(&[uint(84)]).unwrap();
        assert_eq!(witness, uint(5));

        let invalid = goldbach_holds(&[uint(9)]);
        assert!(invalid.is_err());
    }

    #[test]
    fn mersenne_and_lucas_lehmer_workflow() {
        let mersenne = mersenne_number(&[uint(7)]).unwrap();
        assert_eq!(mersenne, uint(127));

        let lucas = lucas_lehmer(&[uint(7)]).unwrap();
        assert_eq!(lucas, bool_value(true));

        let mersenne_prime = is_mersenne_prime(&[uint(7)]).unwrap();
        assert_eq!(mersenne_prime, bool_value(true));

        let mersenne_false = is_mersenne_prime(&[uint(11)]).unwrap();
        assert_eq!(mersenne_false, bool_value(false));

        let lucas_err = lucas_lehmer(&[uint(1)]);
        assert!(lucas_err.is_err());
    }

    #[test]
    fn pseudoprime_classifiers_handle_edge_cases() {
        let fermat = is_fermat_pseudoprime(&[uint(561), uint(2)]).unwrap();
        assert_eq!(fermat, bool_value(true));

        let fermat_fail = is_fermat_pseudoprime(&[uint(15), uint(2)]).unwrap();
        assert_eq!(fermat_fail, bool_value(false));

        let strong_fail = is_strong_pseudoprime(&[uint(561), uint(2)]).unwrap();
        assert_eq!(strong_fail, bool_value(false));

        let strong_pass = is_strong_pseudoprime(&[uint(2047), uint(2)]).unwrap();
        assert_eq!(strong_pass, bool_value(true));

        let invalid_base = is_fermat_pseudoprime(&[uint(561), uint(561)]);
        assert!(invalid_base.is_err());
    }

    #[test]
    fn miller_rabin_rounds_detect_composites() {
        let composite = miller_rabin_test(&[uint(1_373_653), uint(3)]).unwrap();
        assert_eq!(composite, bool_value(false));

        let prime = miller_rabin_test(&[uint(104_729), uint(5)]).unwrap();
        assert_eq!(prime, bool_value(true));

        let zero_rounds = miller_rabin_test(&[uint(17), uint(0)]);
        assert!(zero_rounds.is_err());
    }

    #[test]
    fn carmichael_predicate_matches_classical_examples() {
        let carmichael_true = is_carmichael(&[uint(561)]).unwrap();
        assert_eq!(carmichael_true, bool_value(true));

        let carmichael_false = is_carmichael(&[uint(45)]).unwrap();
        assert_eq!(carmichael_false, bool_value(false));
    }
}
