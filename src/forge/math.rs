//! Math library for ApexForge NightScript

use rust_decimal::Decimal as RustDecimal;
use num_bigint::BigInt as RustBigInt;
use num_complex::Complex64;
use num_rational::Rational64;
use num_traits::{Zero, Signed, ToPrimitive};
use num_integer::{gcd, lcm};
use std::str::FromStr;

/// Decimal type for precise decimal arithmetic
pub type Decimal = RustDecimal;

/// BigInt type for arbitrary precision integers
pub type BigInt = RustBigInt;

/// Complex type for complex number arithmetic
pub type Complex = Complex64;

/// Rational type for exact rational arithmetic
pub type Rational = Rational64;

/// Enhanced Decimal implementation with AFNS methods
pub struct AFNSDecimal {
    value: Decimal,
}

impl AFNSDecimal {
    pub fn new(value: &str) -> Result<Self, String> {
        match Decimal::from_str(value) {
            Ok(decimal) => Ok(AFNSDecimal { value: decimal }),
            Err(e) => Err(format!("Invalid decimal: {}", e)),
        }
    }

    pub fn add(&self, other: &AFNSDecimal) -> AFNSDecimal {
        AFNSDecimal { value: self.value + other.value }
    }

    pub fn subtract(&self, other: &AFNSDecimal) -> AFNSDecimal {
        AFNSDecimal { value: self.value - other.value }
    }

    pub fn multiply(&self, other: &AFNSDecimal) -> AFNSDecimal {
        AFNSDecimal { value: self.value * other.value }
    }

    pub fn divide(&self, other: &AFNSDecimal) -> Result<AFNSDecimal, String> {
        if other.value.is_zero() {
            Err("Division by zero".to_string())
        } else {
            Ok(AFNSDecimal { value: self.value / other.value })
        }
    }

    pub fn round(&self, places: u32) -> AFNSDecimal {
        AFNSDecimal { value: self.value.round_dp(places) }
    }

    pub fn floor(&self) -> AFNSDecimal {
        AFNSDecimal { value: self.value.floor() }
    }

    pub fn ceil(&self) -> AFNSDecimal {
        AFNSDecimal { value: self.value.ceil() }
    }

    pub fn trunc(&self) -> AFNSDecimal {
        AFNSDecimal { value: self.value.trunc() }
    }

    pub fn abs(&self) -> AFNSDecimal {
        AFNSDecimal { value: self.value.abs() }
    }

    pub fn sign(&self) -> i32 {
        if self.value.is_sign_positive() { 1 }
        else if self.value.is_sign_negative() { -1 }
        else { 0 }
    }

    pub fn pow(&self, exponent: u32) -> AFNSDecimal {
        AFNSDecimal { value: self.value.pow(exponent) }
    }

    pub fn sqrt(&self) -> Result<AFNSDecimal, String> {
        if self.value.is_sign_negative() {
            Err("Square root of negative number".to_string())
        } else {
            // Simple approximation for square root
            let two = Decimal::from(2);
            let mut x = self.value;
            for _ in 0..10 {
                x = (x + self.value / x) / two;
            }
            Ok(AFNSDecimal { value: x })
        }
    }

    pub fn parse(value: &str) -> Result<AFNSDecimal, String> {
        AFNSDecimal::new(value)
    }

    pub fn to_string(&self) -> String {
        self.value.to_string()
    }

    pub fn clone(&self) -> AFNSDecimal {
        AFNSDecimal { value: self.value }
    }

    pub fn copy(&self) -> AFNSDecimal {
        self.clone()
    }
}

/// Enhanced BigInt implementation with AFNS methods
pub struct AFNSBigInt {
    value: BigInt,
}

impl AFNSBigInt {
    pub fn new(value: &str) -> Result<Self, String> {
        match BigInt::from_str(value) {
            Ok(bigint) => Ok(AFNSBigInt { value: bigint }),
            Err(e) => Err(format!("Invalid BigInt: {}", e)),
        }
    }

    pub fn add(&self, other: &AFNSBigInt) -> AFNSBigInt {
        AFNSBigInt { value: &self.value + &other.value }
    }

    pub fn subtract(&self, other: &AFNSBigInt) -> AFNSBigInt {
        AFNSBigInt { value: &self.value - &other.value }
    }

    pub fn multiply(&self, other: &AFNSBigInt) -> AFNSBigInt {
        AFNSBigInt { value: &self.value * &other.value }
    }

    pub fn divide(&self, other: &AFNSBigInt) -> Result<AFNSBigInt, String> {
        if other.value.is_zero() {
            Err("Division by zero".to_string())
        } else {
            Ok(AFNSBigInt { value: &self.value / &other.value })
        }
    }

    pub fn modulo(&self, other: &AFNSBigInt) -> Result<AFNSBigInt, String> {
        if other.value.is_zero() {
            Err("Modulo by zero".to_string())
        } else {
            Ok(AFNSBigInt { value: &self.value % &other.value })
        }
    }

    pub fn is_even(&self) -> bool {
        &self.value % 2 == BigInt::from(0)
    }

    pub fn is_odd(&self) -> bool {
        &self.value % 2 != BigInt::from(0)
    }

    pub fn is_prime(&self) -> bool {
        if self.value <= BigInt::from(1) {
            return false;
        }
        if self.value <= BigInt::from(3) {
            return true;
        }
        if &self.value % 2 == 0 || &self.value % 3 == 0 {
            return false;
        }
        
        let mut i = BigInt::from(5);
        while &i * &i <= self.value {
            if &self.value % &i == BigInt::from(0) || &self.value % (&i + BigInt::from(2)) == BigInt::from(0) {
                return false;
            }
            i += 6;
        }
        true
    }

    pub fn gcd(&self, other: &AFNSBigInt) -> AFNSBigInt {
        AFNSBigInt { value: gcd(&self.value, &other.value) }
    }

    pub fn lcm(&self, other: &AFNSBigInt) -> AFNSBigInt {
        AFNSBigInt { value: lcm(&self.value, &other.value) }
    }

    pub fn pow(&self, exponent: u32) -> AFNSBigInt {
        AFNSBigInt { value: self.value.pow(exponent) }
    }

    pub fn sqrt(&self) -> Result<AFNSBigInt, String> {
        if self.value.is_sign_negative() {
            Err("Square root of negative number".to_string())
        } else {
            // Simple approximation for square root
            let mut x = self.value.clone();
            let two = BigInt::from(2);
            for _ in 0..10 {
                x = (&x + &self.value / &x) / &two;
            }
            Ok(AFNSBigInt { value: x })
        }
    }

    pub fn abs(&self) -> AFNSBigInt {
        AFNSBigInt { value: self.value.abs() }
    }

    pub fn sign(&self) -> i32 {
        if self.value.is_sign_positive() { 1 }
        else if self.value.is_sign_negative() { -1 }
        else { 0 }
    }

    pub fn to_binary(&self) -> String {
        format!("{:b}", self.value)
    }

    pub fn to_hex(&self) -> String {
        format!("{:x}", self.value)
    }

    pub fn to_octal(&self) -> String {
        format!("{:o}", self.value)
    }

    pub fn parse(value: &str) -> Result<AFNSBigInt, String> {
        AFNSBigInt::new(value)
    }

    pub fn to_string(&self) -> String {
        self.value.to_string()
    }

    pub fn clone(&self) -> AFNSBigInt {
        AFNSBigInt { value: self.value.clone() }
    }

    pub fn copy(&self) -> AFNSBigInt {
        self.clone()
    }
}

/// Enhanced Complex implementation with AFNS methods
pub struct AFNSComplex {
    value: Complex,
}

impl AFNSComplex {
    pub fn new(real: f64, imag: f64) -> Self {
        AFNSComplex { value: Complex::new(real, imag) }
    }

    pub fn add(&self, other: &AFNSComplex) -> AFNSComplex {
        AFNSComplex { value: self.value + other.value }
    }

    pub fn subtract(&self, other: &AFNSComplex) -> AFNSComplex {
        AFNSComplex { value: self.value - other.value }
    }

    pub fn multiply(&self, other: &AFNSComplex) -> AFNSComplex {
        AFNSComplex { value: self.value * other.value }
    }

    pub fn divide(&self, other: &AFNSComplex) -> Result<AFNSComplex, String> {
        if other.value.norm() == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(AFNSComplex { value: self.value / other.value })
        }
    }

    pub fn real(&self) -> f64 {
        self.value.re
    }

    pub fn imag(&self) -> f64 {
        self.value.im
    }

    pub fn magnitude(&self) -> f64 {
        self.value.norm()
    }

    pub fn phase(&self) -> f64 {
        self.value.arg()
    }

    pub fn conjugate(&self) -> AFNSComplex {
        AFNSComplex { value: self.value.conj() }
    }

    pub fn pow(&self, exponent: u32) -> AFNSComplex {
        AFNSComplex { value: self.value.powi(exponent as i32) }
    }

    pub fn sqrt(&self) -> AFNSComplex {
        AFNSComplex { value: self.value.sqrt() }
    }

    pub fn exp(&self) -> AFNSComplex {
        AFNSComplex { value: self.value.exp() }
    }

    pub fn ln(&self) -> AFNSComplex {
        AFNSComplex { value: self.value.ln() }
    }

    pub fn sin(&self) -> AFNSComplex {
        AFNSComplex { value: self.value.sin() }
    }

    pub fn cos(&self) -> AFNSComplex {
        AFNSComplex { value: self.value.cos() }
    }

    pub fn tan(&self) -> AFNSComplex {
        AFNSComplex { value: self.value.tan() }
    }

    pub fn parse(value: &str) -> Result<AFNSComplex, String> {
        // Simple parsing for "real+imag*i" format
        let parts: Vec<&str> = value.split('+').collect();
        if parts.len() != 2 {
            return Err("Invalid complex format".to_string());
        }
        
        let real_part = parts[0].trim();
        let imag_part = parts[1].trim().trim_end_matches('i');
        
        let real = real_part.parse::<f64>().map_err(|e| format!("Invalid real part: {}", e))?;
        let imag = imag_part.parse::<f64>().map_err(|e| format!("Invalid imaginary part: {}", e))?;
        
        Ok(AFNSComplex::new(real, imag))
    }

    pub fn to_string(&self) -> String {
        format!("{}+{}i", self.value.re, self.value.im)
    }

    pub fn clone(&self) -> AFNSComplex {
        AFNSComplex { value: self.value }
    }

    pub fn copy(&self) -> AFNSComplex {
        self.clone()
    }
}

/// Enhanced Rational implementation with AFNS methods
pub struct AFNSRational {
    value: Rational,
}

impl AFNSRational {
    pub fn new(numerator: i64, denominator: i64) -> Result<Self, String> {
        if denominator == 0 {
            Err("Division by zero".to_string())
        } else {
            Ok(AFNSRational { value: Rational::new(numerator, denominator) })
        }
    }

    pub fn add(&self, other: &AFNSRational) -> AFNSRational {
        AFNSRational { value: self.value + other.value }
    }

    pub fn subtract(&self, other: &AFNSRational) -> AFNSRational {
        AFNSRational { value: self.value - other.value }
    }

    pub fn multiply(&self, other: &AFNSRational) -> AFNSRational {
        AFNSRational { value: self.value * other.value }
    }

    pub fn divide(&self, other: &AFNSRational) -> Result<AFNSRational, String> {
        if other.value.is_zero() {
            Err("Division by zero".to_string())
        } else {
            Ok(AFNSRational { value: self.value / other.value })
        }
    }

    pub fn numerator(&self) -> i64 {
        *self.value.numer()
    }

    pub fn denominator(&self) -> i64 {
        *self.value.denom()
    }

    pub fn to_float(&self) -> f64 {
        self.value.to_f64().unwrap_or(0.0)
    }

    pub fn to_decimal(&self) -> Result<AFNSDecimal, String> {
        AFNSDecimal::new(&self.value.to_f64().unwrap_or(0.0).to_string())
    }

    pub fn simplify(&self) -> AFNSRational {
        AFNSRational { value: self.value.reduced() }
    }

    pub fn reciprocal(&self) -> Result<AFNSRational, String> {
        if self.value.is_zero() {
            Err("Reciprocal of zero".to_string())
        } else {
            Ok(AFNSRational { value: Rational::new(*self.value.denom(), *self.value.numer()) })
        }
    }

    pub fn pow(&self, exponent: u32) -> AFNSRational {
        AFNSRational { value: self.value.pow(exponent as i32) }
    }

    pub fn abs(&self) -> AFNSRational {
        AFNSRational { value: self.value.abs() }
    }

    pub fn sign(&self) -> i32 {
        if self.value.is_sign_positive() { 1 }
        else if self.value.is_sign_negative() { -1 }
        else { 0 }
    }

    pub fn parse(value: &str) -> Result<AFNSRational, String> {
        // Parse "numerator/denominator" format
        let parts: Vec<&str> = value.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid rational format".to_string());
        }
        
        let numerator = parts[0].trim().parse::<i64>().map_err(|e| format!("Invalid numerator: {}", e))?;
        let denominator = parts[1].trim().parse::<i64>().map_err(|e| format!("Invalid denominator: {}", e))?;
        
        AFNSRational::new(numerator, denominator)
    }

    pub fn to_string(&self) -> String {
        format!("{}/{}", self.value.numer(), self.value.denom())
    }

    pub fn clone(&self) -> AFNSRational {
        AFNSRational { value: self.value }
    }

    pub fn copy(&self) -> AFNSRational {
        self.clone()
    }
}
