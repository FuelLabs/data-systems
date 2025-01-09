use fuel_streams_macros::subject::*;

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "test.>"]
#[subject_format = "test.{field1}.{field2}.{field3}"]
struct TestSubject {
    pub field1: Option<String>,
    pub field2: Option<u32>,
    pub field3: Option<String>,
}

#[test]
fn subject_derive_parse() {
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: Some(55),
        field3: Some("bar".to_string()),
    };

    assert_eq!(TestSubject::WILDCARD, "test.>");
    assert_eq!(subject.parse(), "test.foo.55.bar");
}

#[test]
fn subject_derive_wildcard() {
    let wildcard = TestSubject::wildcard(None, Some(10), None);
    assert_eq!(wildcard, "test.*.10.*");
}

#[test]
fn subject_derive_build() {
    let subject =
        TestSubject::build(Some("foo".into()), Some(55), Some("bar".into()));
    assert_eq!(subject.parse(), "test.foo.55.bar");
}

#[test]
fn subject_derive_builder() {
    let subject = TestSubject::new()
        .with_field1(Some("foo".into()))
        .with_field2(Some(55))
        .with_field3(Some("bar".into()));
    assert_eq!(subject.parse(), "test.foo.55.bar");
}

#[test]
fn subject_derive_to_string() {
    let subject = TestSubject::new().with_field1(Some("foo".into()));
    assert_eq!(&subject.to_string(), "test.foo.*.*")
}

#[test]
fn subject_derive_sql_where_exact_match() {
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: Some(55),
        field3: Some("bar".to_string()),
    };

    assert_eq!(subject.parse(), "test.foo.55.bar");
    assert_eq!(
        subject.to_sql_where(),
        "field1 = 'foo' AND field2 = '55' AND field3 = 'bar'"
    );
}

#[test]
fn subject_derive_sql_where_wildcards() {
    let subject = TestSubject {
        field1: None,
        field2: Some(55),
        field3: Some("bar".to_string()),
    };

    assert_eq!(subject.parse(), "test.*.55.bar");
    assert_eq!(subject.to_sql_where(), "field2 = '55' AND field3 = 'bar'");
}

#[test]
fn subject_derive_sql_where_greater_than() {
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: None,
        field3: Some("bar".to_string()),
    };

    assert_eq!(subject.to_sql_where(), "field1 = 'foo' AND field3 = 'bar'");
}

#[test]
fn subject_derive_sql_where_table_only() {
    let subject = TestSubject {
        field1: None,
        field2: None,
        field3: None,
    };

    assert_eq!(subject.parse(), "test.>");
    assert_eq!(subject.to_sql_where(), "TRUE");

    let subject2 = TestSubject::default();
    assert_eq!(subject2.parse(), "test.>");
    assert_eq!(subject2.to_sql_where(), "TRUE");

    let subject3 = TestSubject::new();
    assert_eq!(subject3.parse(), "test.>");
    assert_eq!(subject3.to_sql_where(), "TRUE");
}

#[test]
fn subject_derive_validate_pattern() {
    let subject = TestSubject::new();
    assert!(subject.validate_pattern("test.foo.bar").is_ok());
    assert!(subject.validate_pattern("test.*.55").is_ok());
    assert!(subject.validate_pattern("test.*.*").is_ok());
    assert!(subject.validate_pattern("test.>").is_ok());

    assert!(subject.validate_pattern("test.>.foo").is_err());
    assert!(subject.validate_pattern("test.*.>").is_err());
    assert!(subject.validate_pattern("test.foo.55.>").is_err());
}

#[test]
fn subject_derive_from_json() {
    // Test with all fields
    let subject = TestSubject::from_json_str(
        r#"{"field1": "foo", "field2": 55, "field3": "bar"}"#,
    )
    .unwrap();
    assert_eq!(subject.parse(), "test.foo.55.bar");

    // Test with partial fields
    let subject = TestSubject::from_json_str(r#"{"field1": "foo"}"#).unwrap();
    assert_eq!(subject.parse(), "test.foo.*.*");

    // Test with empty object
    let subject = TestSubject::from_json_str("{}").unwrap();
    assert_eq!(subject.parse(), "test.>");
}

#[test]
fn subject_derive_from_json_error() {
    // Test error cases
    let invalid_json = TestSubject::from_json_str("{invalid}");
    assert!(matches!(
        invalid_json,
        Err(SubjectError::InvalidJsonConversion(_))
    ));

    let invalid_type = TestSubject::from_json_str("[1, 2, 3]");
    assert!(matches!(
        invalid_type,
        Err(SubjectError::ExpectedJsonObject)
    ));
}
