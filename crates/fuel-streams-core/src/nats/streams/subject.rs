use std::fmt;

pub(super) fn parse_param<V: ToString>(param: &Option<V>) -> String {
    match param {
        Some(val) => val.to_string(),
        None => "*".to_string(),
    }
}

pub trait Subject: fmt::Debug + Clone + Default {
    const WILDCARD: &'static str;

    fn wildcard(&self) -> &'static str {
        Self::WILDCARD
    }

    fn parse(&self) -> String;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Debug, Clone, Default)]
    struct TestSubject {
        foo: Option<&'static str>,
    }

    impl Subject for TestSubject {
        const WILDCARD: &'static str = "blocks.*.*";

        fn parse(&self) -> String {
            let foo = parse_param(&self.foo);
            format!("test.{foo}")
        }
    }

    #[test]
    fn parse_wildcard() {
        let subject = TestSubject::default();
        let parsed = subject.parse().to_string();
        assert_eq!(parsed, "test.*")
    }

    #[test]
    fn parse_values() {
        let subject = TestSubject { foo: Some("bar") };
        let parsed = subject.parse().to_string();
        assert_eq!(parsed, "test.bar")
    }
}
