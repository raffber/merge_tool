use std::{fs, path::Path, str::FromStr};

use regex::Regex;
use semver::Version;

use crate::Error;

pub fn extract_version_from_changelog(changelog: &str) -> Result<Version, Error> {
    for line in changelog.lines() {
        let line = line.trim();
        if line.starts_with("## ") {
            return parse_version_from_changelog_line(line);
        }
    }
    Err(Error::CannotParseChangelog)
}

pub fn extract_version_from_changelog_file(fpath: &Path) -> Result<Version, Error> {
    let changelog = fs::read_to_string(fpath)?;
    extract_version_from_changelog(&changelog)
}

fn parse_version_from_changelog_line(line: &str) -> Result<Version, Error> {
    // line is prepended by "## " and version is between [ ] and is of the form "X.Y.Z"
    // use a regex to extract the version
    let regex = Regex::new(r"## \[(?P<version>.*?)\]").unwrap();

    let caps = regex.captures(line).ok_or(Error::CannotParseChangelog)?;
    let version = caps
        .name("version")
        .ok_or(Error::CannotParseChangelog)?
        .as_str();
    Version::from_str(version).map_err(|_| Error::CannotParseChangelog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_from_changelog() {
        let changelog = r#" ## [0.1.0] - 2020-04-01"#;
        let version = extract_version_from_changelog(changelog).unwrap();
        assert_eq!(version, Version::new(0, 1, 0));
    }
}
