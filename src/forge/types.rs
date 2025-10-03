//! Types library for AFNS
//! 
//! This module provides specialized types including:
//! - UUID: Universally unique identifier
//! - Email: Email address validation and handling
//! - URL: URL parsing and validation
//! - IPAddress: IP address (IPv4 and IPv6) handling
//! - MACAddress: MAC address handling
//! - Date: Date handling with timezone support
//! - Duration: Time duration handling

use std::fmt;
use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use uuid::Uuid;
use chrono::{DateTime, Utc, Local, TimeZone, NaiveDate, NaiveDateTime, Duration as ChronoDuration};

/// Universally unique identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AFNSUUID {
    inner: Uuid,
}

impl AFNSUUID {
    /// Create a new random UUID
    pub fn new() -> Self {
        Self { inner: Uuid::new_v4() }
    }

    /// Create a UUID from a string
    pub fn from_string(s: &str) -> Result<Self, String> {
        Uuid::parse_str(s)
            .map(|uuid| Self { inner: uuid })
            .map_err(|e| format!("Invalid UUID string: {}", e))
    }

    /// Create a UUID from bytes
    pub fn from_bytes(bytes: [u8; 16]) -> Result<Self, String> {
        Uuid::from_bytes(bytes)
            .map(|uuid| Self { inner: uuid })
            .map_err(|e| format!("Invalid UUID bytes: {}", e))
    }

    /// Get the UUID as a string
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Get the UUID as a hyphenated string
    pub fn to_hyphenated_string(&self) -> String {
        self.inner.hyphenated().to_string()
    }

    /// Get the UUID as a simple string (no hyphens)
    pub fn to_simple_string(&self) -> String {
        self.inner.simple().to_string()
    }

    /// Get the UUID as bytes
    pub fn as_bytes(&self) -> &[u8; 16] {
        self.inner.as_bytes()
    }

    /// Check if the UUID is nil (all zeros)
    pub fn is_nil(&self) -> bool {
        self.inner.is_nil()
    }

    /// Get the version of the UUID
    pub fn version(&self) -> Option<u8> {
        self.inner.get_version_num()
    }

    /// Get the variant of the UUID
    pub fn variant(&self) -> Option<u8> {
        self.inner.get_variant()
    }
}

impl fmt::Display for AFNSUUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// Email address with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AFNSEmail {
    address: String,
}

impl AFNSEmail {
    /// Create a new email from a string
    pub fn new(address: String) -> Result<Self, String> {
        if Self::is_valid(&address) {
            Ok(Self { address })
        } else {
            Err(format!("Invalid email address: {}", address))
        }
    }

    /// Check if an email address is valid
    pub fn is_valid(email: &str) -> bool {
        // Basic email validation
        email.contains('@') && 
        email.split('@').count() == 2 &&
        !email.starts_with('@') &&
        !email.ends_with('@') &&
        email.len() > 3
    }

    /// Get the email address as a string
    pub fn to_string(&self) -> String {
        self.address.clone()
    }

    /// Get the local part (before @)
    pub fn local_part(&self) -> String {
        self.address.split('@').next().unwrap_or("").to_string()
    }

    /// Get the domain part (after @)
    pub fn domain(&self) -> String {
        self.address.split('@').nth(1).unwrap_or("").to_string()
    }

    /// Check if the email is from a specific domain
    pub fn is_from_domain(&self, domain: &str) -> bool {
        self.domain() == domain
    }
}

impl fmt::Display for AFNSEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

/// URL with parsing and validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AFNSURL {
    url: String,
    scheme: String,
    host: String,
    port: Option<u16>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl AFNSURL {
    /// Create a new URL from a string
    pub fn new(url: String) -> Result<Self, String> {
        Self::parse(&url)
    }

    /// Parse a URL string
    pub fn parse(url: &str) -> Result<Self, String> {
        // Basic URL parsing
        if let Some(scheme_end) = url.find("://") {
            let scheme = url[..scheme_end].to_string();
            let rest = &url[scheme_end + 3..];
            
            let (host_port, path_query_fragment) = if let Some(slash_pos) = rest.find('/') {
                (rest[..slash_pos].to_string(), rest[slash_pos..].to_string())
            } else {
                (rest.to_string(), "/".to_string())
            };

            let (host, port) = if let Some(colon_pos) = host_port.rfind(':') {
                let host_part = host_port[..colon_pos].to_string();
                if let Ok(port_num) = host_port[colon_pos + 1..].parse::<u16>() {
                    (host_part, Some(port_num))
                } else {
                    (host_port, None)
                }
            } else {
                (host_port, None)
            };

            let (path, query_fragment) = if let Some(q_pos) = path_query_fragment.find('?') {
                (path_query_fragment[..q_pos].to_string(), path_query_fragment[q_pos + 1..].to_string())
            } else {
                (path_query_fragment, String::new())
            };

            let (query, fragment) = if let Some(f_pos) = query_fragment.find('#') {
                (Some(query_fragment[..f_pos].to_string()), Some(query_fragment[f_pos + 1..].to_string()))
            } else if !query_fragment.is_empty() {
                (Some(query_fragment), None)
            } else {
                (None, None)
            };

            Ok(Self {
                url: url.to_string(),
                scheme,
                host,
                port,
                path,
                query,
                fragment,
            })
        } else {
            Err(format!("Invalid URL format: {}", url))
        }
    }

    /// Get the full URL as a string
    pub fn to_string(&self) -> String {
        self.url.clone()
    }

    /// Get the scheme (e.g., http, https)
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// Get the host
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Get the port
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Get the path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the query string
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Get the fragment
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    /// Check if the URL is secure (https)
    pub fn is_secure(&self) -> bool {
        self.scheme == "https"
    }

    /// Get the default port for the scheme
    pub fn default_port(&self) -> Option<u16> {
        match self.scheme.as_str() {
            "http" => Some(80),
            "https" => Some(443),
            "ftp" => Some(21),
            "ssh" => Some(22),
            "telnet" => Some(23),
            "smtp" => Some(25),
            "dns" => Some(53),
            "pop3" => Some(110),
            "nntp" => Some(119),
            "imap" => Some(143),
            "snmp" => Some(161),
            "ldap" => Some(389),
            "https" => Some(443),
            "smtps" => Some(465),
            "imaps" => Some(993),
            "pop3s" => Some(995),
            _ => None,
        }
    }
}

impl fmt::Display for AFNSURL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

/// IP address (IPv4 and IPv6)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AFNSIPAddress {
    inner: IpAddr,
}

impl AFNSIPAddress {
    /// Create a new IP address from a string
    pub fn new(addr: String) -> Result<Self, String> {
        addr.parse::<IpAddr>()
            .map(|ip| Self { inner: ip })
            .map_err(|e| format!("Invalid IP address: {}", e))
    }

    /// Create an IPv4 address
    pub fn ipv4(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self { inner: IpAddr::V4(Ipv4Addr::new(a, b, c, d)) }
    }

    /// Create an IPv6 address
    pub fn ipv6(segments: [u16; 8]) -> Self {
        Self { inner: IpAddr::V6(Ipv6Addr::new(
            segments[0], segments[1], segments[2], segments[3],
            segments[4], segments[5], segments[6], segments[7],
        ))}
    }

    /// Get the IP address as a string
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Check if the IP is IPv4
    pub fn is_ipv4(&self) -> bool {
        matches!(self.inner, IpAddr::V4(_))
    }

    /// Check if the IP is IPv6
    pub fn is_ipv6(&self) -> bool {
        matches!(self.inner, IpAddr::V6(_))
    }

    /// Check if the IP is a loopback address
    pub fn is_loopback(&self) -> bool {
        self.inner.is_loopback()
    }

    /// Check if the IP is a private address
    pub fn is_private(&self) -> bool {
        self.inner.is_private()
    }

    /// Check if the IP is a multicast address
    pub fn is_multicast(&self) -> bool {
        self.inner.is_multicast()
    }

    /// Check if the IP is a link-local address
    pub fn is_link_local(&self) -> bool {
        self.inner.is_link_local()
    }

    /// Check if the IP is a global address
    pub fn is_global(&self) -> bool {
        self.inner.is_global()
    }

    /// Get the IP as bytes
    pub fn as_bytes(&self) -> Vec<u8> {
        match self.inner {
            IpAddr::V4(ipv4) => ipv4.octets().to_vec(),
            IpAddr::V6(ipv6) => ipv6.octets().to_vec(),
        }
    }
}

impl fmt::Display for AFNSIPAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// MAC address
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AFNSMACAddress {
    bytes: [u8; 6],
}

impl AFNSMACAddress {
    /// Create a new MAC address from bytes
    pub fn new(bytes: [u8; 6]) -> Self {
        Self { bytes }
    }

    /// Create a MAC address from a string
    pub fn from_string(s: &str) -> Result<Self, String> {
        let cleaned = s.replace([':', '-', '.'], "");
        if cleaned.len() != 12 {
            return Err(format!("Invalid MAC address format: {}", s));
        }

        let mut bytes = [0u8; 6];
        for (i, chunk) in cleaned.chars().collect::<Vec<_>>().chunks(2).enumerate() {
            if i >= 6 {
                break;
            }
            let hex_str = format!("{}{}", chunk[0], chunk[1]);
            bytes[i] = u8::from_str_radix(&hex_str, 16)
                .map_err(|_| format!("Invalid hex in MAC address: {}", s))?;
        }

        Ok(Self { bytes })
    }

    /// Get the MAC address as a string
    pub fn to_string(&self) -> String {
        format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                self.bytes[0], self.bytes[1], self.bytes[2],
                self.bytes[3], self.bytes[4], self.bytes[5])
    }

    /// Get the MAC address as bytes
    pub fn as_bytes(&self) -> [u8; 6] {
        self.bytes
    }

    /// Check if the MAC address is a broadcast address
    pub fn is_broadcast(&self) -> bool {
        self.bytes == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    /// Check if the MAC address is a multicast address
    pub fn is_multicast(&self) -> bool {
        self.bytes[0] & 0x01 != 0
    }

    /// Check if the MAC address is a unicast address
    pub fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }

    /// Check if the MAC address is locally administered
    pub fn is_locally_administered(&self) -> bool {
        self.bytes[0] & 0x02 != 0
    }

    /// Check if the MAC address is universally administered
    pub fn is_universally_administered(&self) -> bool {
        !self.is_locally_administered()
    }
}

impl fmt::Display for AFNSMACAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Date with timezone support
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AFNSDate {
    inner: DateTime<Utc>,
}

impl AFNSDate {
    /// Create a new date from a string
    pub fn new(date_str: String) -> Result<Self, String> {
        // Try parsing various date formats
        let formats = [
            "%Y-%m-%d",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S%.fZ",
        ];

        for format in &formats {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&date_str, format) {
                return Ok(Self { inner: Utc.from_utc_datetime(&naive_dt) });
            }
        }

        // Try parsing as date only
        if let Ok(naive_date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            let naive_dt = naive_date.and_hms(0, 0, 0);
            return Ok(Self { inner: Utc.from_utc_datetime(&naive_dt) });
        }

        Err(format!("Invalid date format: {}", date_str))
    }

    /// Create a date from year, month, day
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Result<Self, String> {
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| format!("Invalid date: {}-{}-{}", year, month, day))?;
        let naive_dt = naive_date.and_hms(0, 0, 0);
        Ok(Self { inner: Utc.from_utc_datetime(&naive_dt) })
    }

    /// Create a date from year, month, day, hour, minute, second
    pub fn from_ymdhms(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Result<Self, String> {
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| format!("Invalid date: {}-{}-{}", year, month, day))?;
        let naive_dt = naive_date.and_hms_opt(hour, minute, second)
            .ok_or_else(|| format!("Invalid time: {}:{}:{}", hour, minute, second))?;
        Ok(Self { inner: Utc.from_utc_datetime(&naive_dt) })
    }

    /// Get the current date and time
    pub fn now() -> Self {
        Self { inner: Utc::now() }
    }

    /// Get the date as a string
    pub fn to_string(&self) -> String {
        self.inner.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    /// Get the date in ISO format
    pub fn to_iso_string(&self) -> String {
        self.inner.to_rfc3339()
    }

    /// Get the year
    pub fn year(&self) -> i32 {
        self.inner.year()
    }

    /// Get the month
    pub fn month(&self) -> u32 {
        self.inner.month()
    }

    /// Get the day
    pub fn day(&self) -> u32 {
        self.inner.day()
    }

    /// Get the hour
    pub fn hour(&self) -> u32 {
        self.inner.hour()
    }

    /// Get the minute
    pub fn minute(&self) -> u32 {
        self.inner.minute()
    }

    /// Get the second
    pub fn second(&self) -> u32 {
        self.inner.second()
    }

    /// Get the day of the week (1 = Monday, 7 = Sunday)
    pub fn weekday(&self) -> u32 {
        self.inner.weekday().number_from_monday()
    }

    /// Get the day of the year
    pub fn day_of_year(&self) -> u32 {
        self.inner.ordinal()
    }

    /// Add a duration to the date
    pub fn add_duration(&self, duration: &AFNSDuration) -> Self {
        Self { inner: self.inner + duration.inner }
    }

    /// Subtract a duration from the date
    pub fn sub_duration(&self, duration: &AFNSDuration) -> Self {
        Self { inner: self.inner - duration.inner }
    }

    /// Get the difference between two dates
    pub fn diff(&self, other: &AFNSDate) -> AFNSDuration {
        AFNSDuration { inner: self.inner.signed_duration_since(other.inner) }
    }

    /// Check if the date is in the past
    pub fn is_past(&self) -> bool {
        self.inner < Utc::now()
    }

    /// Check if the date is in the future
    pub fn is_future(&self) -> bool {
        self.inner > Utc::now()
    }

    /// Check if the date is today
    pub fn is_today(&self) -> bool {
        let now = Utc::now();
        self.inner.date() == now.date()
    }
}

impl fmt::Display for AFNSDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Time duration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AFNSDuration {
    inner: ChronoDuration,
}

impl AFNSDuration {
    /// Create a duration from seconds
    pub fn from_seconds(seconds: i64) -> Self {
        Self { inner: ChronoDuration::seconds(seconds) }
    }

    /// Create a duration from milliseconds
    pub fn from_milliseconds(milliseconds: i64) -> Self {
        Self { inner: ChronoDuration::milliseconds(milliseconds) }
    }

    /// Create a duration from microseconds
    pub fn from_microseconds(microseconds: i64) -> Self {
        Self { inner: ChronoDuration::microseconds(microseconds) }
    }

    /// Create a duration from nanoseconds
    pub fn from_nanoseconds(nanoseconds: i64) -> Self {
        Self { inner: ChronoDuration::nanoseconds(nanoseconds) }
    }

    /// Create a duration from days
    pub fn from_days(days: i64) -> Self {
        Self { inner: ChronoDuration::days(days) }
    }

    /// Create a duration from hours
    pub fn from_hours(hours: i64) -> Self {
        Self { inner: ChronoDuration::hours(hours) }
    }

    /// Create a duration from minutes
    pub fn from_minutes(minutes: i64) -> Self {
        Self { inner: ChronoDuration::minutes(minutes) }
    }

    /// Create a duration from weeks
    pub fn from_weeks(weeks: i64) -> Self {
        Self { inner: ChronoDuration::weeks(weeks) }
    }

    /// Get the duration in seconds
    pub fn as_seconds(&self) -> i64 {
        self.inner.num_seconds()
    }

    /// Get the duration in milliseconds
    pub fn as_milliseconds(&self) -> i64 {
        self.inner.num_milliseconds()
    }

    /// Get the duration in microseconds
    pub fn as_microseconds(&self) -> i64 {
        self.inner.num_microseconds().unwrap_or(0)
    }

    /// Get the duration in nanoseconds
    pub fn as_nanoseconds(&self) -> i64 {
        self.inner.num_nanoseconds().unwrap_or(0)
    }

    /// Get the duration in days
    pub fn as_days(&self) -> i64 {
        self.inner.num_days()
    }

    /// Get the duration in hours
    pub fn as_hours(&self) -> i64 {
        self.inner.num_hours()
    }

    /// Get the duration in minutes
    pub fn as_minutes(&self) -> i64 {
        self.inner.num_minutes()
    }

    /// Get the duration in weeks
    pub fn as_weeks(&self) -> i64 {
        self.inner.num_weeks()
    }

    /// Add another duration
    pub fn add(&self, other: &AFNSDuration) -> Self {
        Self { inner: self.inner + other.inner }
    }

    /// Subtract another duration
    pub fn sub(&self, other: &AFNSDuration) -> Self {
        Self { inner: self.inner - other.inner }
    }

    /// Multiply the duration by a scalar
    pub fn mul(&self, scalar: i64) -> Self {
        Self { inner: self.inner * scalar }
    }

    /// Divide the duration by a scalar
    pub fn div(&self, scalar: i64) -> Result<Self, String> {
        if scalar == 0 {
            return Err("Cannot divide duration by zero".to_string());
        }
        Ok(Self { inner: self.inner / scalar })
    }

    /// Check if the duration is zero
    pub fn is_zero(&self) -> bool {
        self.inner == ChronoDuration::zero()
    }

    /// Check if the duration is positive
    pub fn is_positive(&self) -> bool {
        self.inner > ChronoDuration::zero()
    }

    /// Check if the duration is negative
    pub fn is_negative(&self) -> bool {
        self.inner < ChronoDuration::zero()
    }

    /// Get the absolute value of the duration
    pub fn abs(&self) -> Self {
        Self { inner: self.inner.abs() }
    }

    /// Get the duration as a human-readable string
    pub fn to_string(&self) -> String {
        let total_seconds = self.inner.num_seconds();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        let mut parts = Vec::new();
        if days > 0 {
            parts.push(format!("{}d", days));
        }
        if hours > 0 {
            parts.push(format!("{}h", hours));
        }
        if minutes > 0 {
            parts.push(format!("{}m", minutes));
        }
        if seconds > 0 || parts.is_empty() {
            parts.push(format!("{}s", seconds));
        }

        parts.join(" ")
    }
}

impl fmt::Display for AFNSDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// Type aliases for common use cases
pub type UUID = AFNSUUID;
pub type Email = AFNSEmail;
pub type URL = AFNSURL;
pub type IPAddress = AFNSIPAddress;
pub type MACAddress = AFNSMACAddress;
pub type Date = AFNSDate;
pub type Duration = AFNSDuration;