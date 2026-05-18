use nutype::nutype;
use url::Url;

// ===== Identifiers =====

#[nutype(
    sanitize(trim),
    validate(not_empty, regex = r"^[A-Za-z0-9\-_.:]+$"),
    derive(
        Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
        Display, AsRef, Deref, FromStr,
    ),
)]
pub struct RepoId(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct RepoName(String);

#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct Username(String);

#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct Password(String);

#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct ProxyUsername(String);

#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct ProxyPassword(String);

#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct UserAgent(String);

#[nutype(
    sanitize(trim),
    derive(Debug, Clone, PartialEq, Eq, Display, AsRef, Deref, FromStr),
)]
pub struct ModulePlatformId(String);

// ===== Numerics =====

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 99),
    default = 99,
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Deref, Default),
)]
pub struct Priority(i32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 1000,
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Deref, Default),
)]
pub struct Cost(i32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 10,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, Default),
)]
pub struct Retries(u32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 30,
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Deref, Default),
)]
pub struct TimeoutSeconds(u32);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 100),
    default = 75,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, Default),
)]
pub struct DeltaRpmPercentage(u32);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 20),
    default = 3,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, Default),
)]
pub struct MaxParallelDownloads(u32);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10),
    default = 2,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, Default),
)]
pub struct DebugLevel(u8);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10),
    default = 9,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, Default),
)]
pub struct LogLevel(u8);

#[nutype(
    validate(greater_or_equal = 0, predicate = |x| *x != 1),
    default = 3,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, Default),
)]
pub struct InstallOnlyLimit(u32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 4,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, Default),
)]
pub struct LogRotate(u32);

#[nutype(
    validate(greater_or_equal = 0),
    default = 10800,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, Default),
)]
pub struct MetadataTimerSync(u32);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10),
    default = 3,
    derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deref, Default),
)]
pub struct ErrorLevel(u8);

// ===== Composite value types =====

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageSize(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataExpire {
    Duration(u64),
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Throttle {
    Absolute(StorageSize),
    Percent(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxySetting {
    Unset,
    Disabled,
    Url(Url),
}

// ===== DNF Boolean =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DnfBool {
    True,
    False,
}

impl DnfBool {
    pub fn parse(s: &str) -> std::result::Result<Self, crate::error::ParseBoolError> {
        let lower: String = s.chars().map(|c| c.to_ascii_lowercase()).collect();
        match lower.as_str() {
            "1" | "yes" | "true" | "on" => Ok(DnfBool::True),
            "0" | "no" | "false" | "off" => Ok(DnfBool::False),
            _ => Err(crate::error::ParseBoolError { input: s.to_owned() }),
        }
    }
}

impl std::fmt::Display for DnfBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnfBool::True => write!(f, "1"),
            DnfBool::False => write!(f, "0"),
        }
    }
}

impl From<bool> for DnfBool {
    fn from(b: bool) -> Self {
        if b { DnfBool::True } else { DnfBool::False }
    }
}

impl From<DnfBool> for bool {
    fn from(d: DnfBool) -> bool {
        matches!(d, DnfBool::True)
    }
}

// ===== Enums =====

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpResolve { V4, V6 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyAuthMethod { Any, None_, Basic, Digest, Negotiate, Ntlm, DigestIe, NtlmWb }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoMetadataType { RpmMd }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultilibPolicy { Best, All }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Persistence { Auto, Transient, Persist }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpmVerbosity { Critical, Emergency, Error, Warn, Info, Debug }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TsFlag { NoScripts, Test, NoTriggers, NoDocs, JustDb, NoContexts, NoCaps, NoCrypto, Deploops, NoPlugins }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlSource {
    BaseUrl(Vec<Url>),
    MirrorList(Url),
    Metalink(Url),
}
