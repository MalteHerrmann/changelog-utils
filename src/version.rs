use crate::errors::VersionError;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8,
    rc_version: Option<u8>,
}

impl Version {
    /// Checks if the version is higher than the other version.
    pub fn gt(&self, other: &Version) -> bool {
        if self.major > other.major {
            return true;
        }

        if self.major < other.major {
            return false;
        }

        if self.minor > other.minor {
            return true;
        }

        if self.minor < other.minor {
            return false;
        }

        if self.patch > other.patch {
            return true;
        }

        if self.patch < other.patch {
            return false;
        }

        match self.rc_version {
            Some(v) => match other.rc_version {
                Some(v_other) => v > v_other,
                None => false,
            },
            // NOTE: if self is not an rc, but other is -> self is greater
            None => other.rc_version.is_some(),
        }
    }
}

/// Tries to parse the given version string.
/// Returns an instance of Version, in case a valid version is passed.
pub fn parse(version: &str) -> Result<Version, VersionError> {
    let captures = match Regex::new(concat!(
        r"^v(?P<major>\d+)\.",
        r"(?P<minor>\d+)\.",
        r"(?P<patch>\d+)",
        r"(-rc(?P<rc>\d+))*$"
    ))?
    .captures(version)
    {
        Some(c) => c,
        None => return Err(VersionError::NoMatchFound),
    };

    let major = captures.name("major").unwrap().as_str().parse::<u8>()?;
    let minor = captures.name("minor").unwrap().as_str().parse::<u8>()?;
    let patch = captures.name("patch").unwrap().as_str().parse::<u8>()?;
    let rc_version: Option<u8> = match captures.name("rc") {
        Some(c) => Some(c.as_str().parse::<u8>()?),
        None => None,
    };

    Ok(Version {
        major,
        minor,
        patch,
        rc_version,
    })
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn test_greater() {
        let a = parse("v1.1.1-rc1").expect("failed to parse version");

        assert!(a.gt(&parse("v0.2.0").unwrap()));
        assert!(a.gt(&parse("v0.0.2").unwrap()));
        assert!(a.gt(&parse("v0.0.2-rc2").unwrap()));
        assert!(a.gt(&parse("v1.0.1-rc1").unwrap()));
        assert!(a.gt(&parse("v1.0.2-rc2").unwrap()));
        assert!(a.gt(&parse("v1.0.1-rc2").unwrap()));
        assert!(a.gt(&parse("v1.1.0-rc1").unwrap()));
        assert!(a.gt(&parse("v1.1.1-rc0").unwrap()));

        assert!(!a.gt(&parse("v1.1.1").unwrap()));
        assert!(!a.gt(&parse("v1.1.1-rc2").unwrap()));
        assert!(!a.gt(&parse("v1.1.2-rc1").unwrap()));
        assert!(!a.gt(&parse("v1.2.0").unwrap()));
        assert!(!a.gt(&parse("v1.2.0-rc1").unwrap()));
        assert!(!a.gt(&parse("v2.0.0").unwrap()));
        assert!(!a.gt(&parse("v2.0.0-rc1").unwrap()));
    }

    #[test]
    fn test_is_valid_version_pass() {
        let version = parse("v10.0.2").expect("failed to parse version");
        assert_eq!(version.major, 10);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 2);
        assert!(version.rc_version.is_none());
    }

    #[test]
    fn test_pass_release_candidate() {
        let version =
            parse("v11.0.2-rc1").expect("failed to parse valid release candidate version");
        assert_eq!(version.major, 11);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 2);
        assert!(version.rc_version.is_some());
        assert_eq!(version.rc_version.unwrap(), 1);
    }

    #[test]
    fn test_fail_malformed() {
        assert!(parse("v14.0.").is_err());
        assert!(parse("v.0.1").is_err());
        assert!(parse("v11.0.1rc3").is_err());
    }
}
