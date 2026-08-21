use super::{ValidateError, ValidateResult};

/// Length of a UUID in its canonical form: 32 hexadecimal digits plus the
/// 4 hyphens that group them.
const LENGTH: usize = 36;

/// How many digits each hyphen separated group holds.
const GROUPS: [usize; 5] = [8, 4, 4, 4, 12];

/// Validate a string is a UUID in the canonical hyphenated form, such as
/// `67e55044-10b1-426f-9247-bb680e5fe0c8`. Digits may be upper or lower case.
///
/// Only the shape is checked, not the version and variant bits, so the nil
/// UUID and any non-standard version are both accepted.
pub fn uuid<T: AsRef<str>, D, C>(
    value: &T,
    _data: &D,
    _context: &C,
    _finalize: bool,
) -> ValidateResult {
    let value = value.as_ref();
    let length = value.chars().count();

    if length != LENGTH {
        return Err(ValidateError::new(format!(
            "not a valid UUID: expected {LENGTH} characters, found {length}"
        )));
    }

    let grouping = || {
        ValidateError::new("not a valid UUID: expected 5 groups of 8-4-4-4-12 hexadecimal digits")
    };
    let mut groups = value.split('-');

    for expected in GROUPS {
        let Some(group) = groups.next() else {
            return Err(grouping());
        };

        // Characters are checked before the size so that a stray one is named,
        // rather than being reported as a group of the wrong length
        if let Some(char) = group.chars().find(|char| !char.is_ascii_hexdigit()) {
            return Err(ValidateError::new(format!(
                "not a valid UUID: invalid character `{char}`"
            )));
        }

        // Every character is now known to be ASCII, so bytes count as characters
        if group.len() != expected {
            return Err(grouping());
        }
    }

    // Five groups of the right size, plus their hyphens, account for all 36
    // characters, so there can be no sixth
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(value: &str) -> ValidateResult {
        uuid(&value, &(), &(), false)
    }

    // `Display` prefixes the setting path, which is empty here
    fn message(value: &str) -> String {
        check(value).unwrap_err().message
    }

    #[test]
    fn accepts_canonical_uuids() {
        assert!(check("67e55044-10b1-426f-9247-bb680e5fe0c8").is_ok());
        // v1 through v8, and versions we don't know about, are all shapes
        assert!(check("2c5ea4c0-4067-11e9-8bad-9b1deb4d3b7d").is_ok());
        assert!(check("01890a5d-ac96-774b-bcce-b302099a8057").is_ok());
    }

    #[test]
    fn accepts_either_case() {
        assert!(check("67E55044-10B1-426F-9247-BB680E5FE0C8").is_ok());
        assert!(check("67e55044-10B1-426f-9247-BB680e5fe0c8").is_ok());
    }

    #[test]
    fn accepts_the_nil_and_max_uuids() {
        assert!(check("00000000-0000-0000-0000-000000000000").is_ok());
        assert!(check("ffffffff-ffff-ffff-ffff-ffffffffffff").is_ok());
    }

    #[test]
    fn rejects_the_wrong_length() {
        assert_eq!(
            message(""),
            "not a valid UUID: expected 36 characters, found 0"
        );
        // One digit short
        assert_eq!(
            message("67e55044-10b1-426f-9247-bb680e5fe0c"),
            "not a valid UUID: expected 36 characters, found 35"
        );
        // Counted in characters, not bytes
        assert_eq!(
            message("éé7e55044-10b1-426f-9247-bb680e5fe0c8"),
            "not a valid UUID: expected 36 characters, found 37"
        );
    }

    #[test]
    fn rejects_the_wrong_grouping() {
        let expected = "not a valid UUID: expected 5 groups of 8-4-4-4-12 hexadecimal digits";

        // Hyphens in the wrong places
        assert_eq!(message("67e5504-410b1-426f-9247-bb680e5fe0c8"), expected);
        // No hyphens at all, padded to the right length
        assert_eq!(message("67e5504410b1426f9247bb680e5fe0c8----"), expected);
        // An extra group, at the cost of a digit
        assert_eq!(message("67e55044-10b1-426f-9247-bb680e5fe0-8"), expected);
    }

    #[test]
    fn rejects_non_hexadecimal_digits() {
        assert_eq!(
            message("g7e55044-10b1-426f-9247-bb680e5fe0c8"),
            "not a valid UUID: invalid character `g`"
        );
        // A multi byte character is 1 character, so the length still matches
        assert_eq!(
            message("é7e55044-10b1-426f-9247-bb680e5fe0c8"),
            "not a valid UUID: invalid character `é`"
        );
    }

    #[test]
    fn rejects_other_common_forms() {
        // Braced
        assert!(check("{67e55044-10b1-426f-9247-bb680e5fe0c8}").is_err());
        // URN
        assert!(check("urn:uuid:67e55044-10b1-426f-9247-bb680e5fe0c8").is_err());
        // Simple, without hyphens
        assert!(check("67e5504410b1426f9247bb680e5fe0c8").is_err());
    }
}
