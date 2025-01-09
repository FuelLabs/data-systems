#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum SubjectPatternError {
    #[error("Subject pattern cannot be empty")]
    Empty,
    #[error("Greater than wildcard (>) must be at the end of the pattern")]
    GreaterThanNotAtEnd,
    #[error("Cannot mix wildcards (* and >) in the same pattern")]
    MixedWildcards,
    #[error("Asterisk (*) must be the only character in its segment")]
    InvalidAsteriskUsage,
}

pub struct SubjectValidator;

impl SubjectValidator {
    pub fn validate(pattern: &str) -> Result<(), SubjectPatternError> {
        if pattern.is_empty() {
            return Err(SubjectPatternError::Empty);
        }

        // Check for greater than wildcard (>) validation
        if pattern.contains('>') {
            let has_asterisk = pattern.contains('*');
            let ends_with_greater = pattern.ends_with('>');
            let last_segment = pattern.split('.').last();
            let greater_in_last_segment = matches!(last_segment, Some(">"));

            // If pattern has asterisk, that's an immediate error
            if has_asterisk {
                return Err(SubjectPatternError::MixedWildcards);
            }
            // Greater than must be at the end and be the entire last segment
            if !ends_with_greater || !greater_in_last_segment {
                return Err(SubjectPatternError::GreaterThanNotAtEnd);
            }

            return Ok(());
        }

        if pattern.contains('*') {
            for segment in pattern.split('.') {
                if segment.contains('*') && segment.len() > 1 {
                    return Err(SubjectPatternError::InvalidAsteriskUsage);
                }
            }
        }

        Ok(())
    }

    pub fn is_valid(pattern: &str) -> bool {
        Self::validate(pattern).is_ok()
    }

    pub fn subject_matches(subject: &str, pattern: &str) -> bool {
        let pattern = pattern.replace('%', ".*");
        let re_pattern = format!("^{}$", pattern);
        regex::Regex::new(&re_pattern)
            .map(|re| re.is_match(subject))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("orders.*" ; "valid_1: 'orders.*'")]
    #[test_case("orders.*.new" ; "valid_2: 'orders.*.new'")]
    #[test_case("orders.>" ; "valid_3: 'orders.>'")]
    #[test_case("orders.processed.>" ; "valid_4: 'orders.processed.>'")]
    #[test_case("orders.processed.new" ; "valid_5: 'orders.processed.new'")]
    fn test_valid_patterns(pattern: &str) {
        assert!(
            SubjectValidator::is_valid(pattern),
            "Pattern should be valid: {}",
            pattern
        );
        assert!(
            SubjectValidator::validate(pattern).is_ok(),
            "Pattern should validate: {}",
            pattern
        );
    }

    #[test_case("" => SubjectPatternError::Empty ; "invalid_1: ''")]
    #[test_case("orders.>.123" => SubjectPatternError::GreaterThanNotAtEnd ; "invalid_2: 'orders.>.123'")]
    #[test_case("*>" => SubjectPatternError::MixedWildcards ; "invalid_3: '*>'")]
    #[test_case(">*" => SubjectPatternError::MixedWildcards ; "invalid_4: '>*'")]
    #[test_case("orders.*123>" => SubjectPatternError::MixedWildcards ; "invalid_5: 'orders.*123>'")]
    #[test_case("order.proce>.*" => SubjectPatternError::MixedWildcards ; "invalid_6: 'order.proce>.*'")]
    #[test_case("order.proce*.*" => SubjectPatternError::InvalidAsteriskUsage ; "invalid_7: 'order.proce*.*'")]
    #[test_case("orders.new*" => SubjectPatternError::InvalidAsteriskUsage ; "invalid_8: 'orders.new*'")]
    #[test_case("orders.*new" => SubjectPatternError::InvalidAsteriskUsage ; "invalid_9: 'orders.*new'")]
    #[test_case("orders.pro*.new" => SubjectPatternError::InvalidAsteriskUsage ; "invalid_10: 'orders.pro*.new'")]
    fn test_invalid_patterns(pattern: &str) -> SubjectPatternError {
        assert!(
            !SubjectValidator::is_valid(pattern),
            "Pattern '{}' should be invalid",
            pattern
        );
        SubjectValidator::validate(pattern).unwrap_err()
    }
}
