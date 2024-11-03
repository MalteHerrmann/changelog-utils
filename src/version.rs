use crate::{errors::VersionError, release_type::ReleaseType};
use regex::Regex;
use std::fmt;

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

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut version_string = format!("v{}.{}.{}", self.major, self.minor, self.patch);
        version_string = match self.rc_version {
            Some(rc) => version_string + &format!("-rc{}", rc),
            None => version_string,
        };

        write!(f, "{}", version_string)
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

/// Represents the release type.
/// Increments the version based on the given release type.
pub fn bump_version(version: &Version, release_type: &ReleaseType) -> Version {
    let (major, minor, patch, rc) = match release_type {
        ReleaseType::Major => (version.major + 1, 0, 0, None),
        ReleaseType::Minor => (version.major, version.minor + 1, 0, None),
        ReleaseType::Patch => (version.major, version.minor, version.patch + 1, None),
        ReleaseType::RcMajor => match version.rc_version {
            Some(rc) => (version.major, version.minor, version.patch, Some(rc + 1)),
            None => (version.major + 1, 0, 0, Some(1)),
        },
        ReleaseType::RcMinor => match version.rc_version {
            Some(rc) => (version.major, version.minor, version.patch, Some(rc + 1)),
            None => (version.major, version.minor + 1, 0, Some(1)),
        },
        ReleaseType::RcPatch => match version.rc_version {
            Some(rc) => (version.major, version.minor, version.patch, Some(rc + 1)),
            None => (version.major, version.minor, version.patch + 1, Some(1)),
        },
    };
    Version {
        major,
        minor,
        patch,
        rc_version: rc,
    }
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

    #[test]
    fn test_bump_version_major() {
        let version = parse("v1.2.3").expect("failed to parse version");
        let bumped = bump_version(&version, &ReleaseType::Major);
        assert_eq!(bumped.to_string(), "v2.0.0");
    }

    #[test]
    fn test_bump_version_minor() {
        let version = parse("v1.2.3").expect("failed to parse version");
        let bumped = bump_version(&version, &ReleaseType::Minor);
        assert_eq!(bumped.to_string(), "v1.3.0");
    }

    #[test]
    fn test_bump_version_patch() {
        let version = parse("v1.2.3").expect("failed to parse version");
        let bumped = bump_version(&version, &ReleaseType::Patch);
        assert_eq!(bumped.to_string(), "v1.2.4");
    }

    #[test]
    fn test_bump_version_rc_patch() {
        let version = parse("v1.2.3").expect("failed to parse version");
        let bumped = bump_version(&version, &ReleaseType::RcPatch);
        assert_eq!(bumped.to_string(), "v1.2.4-rc1");
    }

    #[test]
    fn test_bump_version_rc_patch_increment() {
        let version = parse("v1.2.3-rc1").expect("failed to parse version");
        let bumped = bump_version(&version, &ReleaseType::RcPatch);
        assert_eq!(bumped.to_string(), "v1.2.3-rc2");
    }

    #[test]
    fn test_bump_version_rc_major() {
        let version = parse("v1.2.3").expect("failed to parse version");
        let bumped = bump_version(&version, &ReleaseType::RcMajor);
        assert_eq!(bumped.to_string(), "v2.0.0-rc1");
    }

    #[test]
    fn test_bump_version_rc_minor() {
        let version = parse("v1.2.3").expect("failed to parse version");
        let bumped = bump_version(&version, &ReleaseType::RcMinor);
        assert_eq!(bumped.to_string(), "v1.3.0-rc1");
    }
}
