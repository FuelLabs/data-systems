use fuel_streams_macros::subject::*;
use serde::{Deserialize, Serialize};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "test"]
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
fn subject_derive_from_json() {
    // Test with all fields
    let subject = TestSubject::from_json(
        r#"{"field1": "foo", "field2": 55, "field3": "bar"}"#,
    )
    .unwrap();
    assert_eq!(subject.parse(), "test.foo.55.bar");

    // Test with partial fields
    let subject = TestSubject::from_json(r#"{"field1": "foo"}"#).unwrap();
    assert_eq!(subject.parse(), "test.foo.*.*");

    // Test with empty object
    let subject = TestSubject::from_json("{}").unwrap();
    assert_eq!(subject.parse(), "test.>");
}

#[test]
fn subject_derive_from_json_error() {
    // Test error cases
    let invalid_json = TestSubject::from_json("{invalid}");
    assert!(matches!(
        invalid_json,
        Err(SubjectError::InvalidJsonConversion(_))
    ));

    let invalid_type = TestSubject::from_json("[1, 2, 3]");
    assert!(matches!(
        invalid_type,
        Err(SubjectError::ExpectedJsonObject)
    ));
}

#[test]
fn subject_derive_id() {
    let subject = TestSubject::new();
    assert_eq!(TestSubject::ID, "test");
    assert_eq!(subject.id(), "test");
}

#[test]
fn subject_derive_to_json() {
    // Test with all fields
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: Some(55),
        field3: Some("bar".to_string()),
    };
    assert_eq!(
        subject.to_json(),
        r#"{"field1":"foo","field2":55,"field3":"bar"}"#
    );

    // Test with partial fields
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: None,
        field3: None,
    };
    assert_eq!(
        subject.to_json(),
        r#"{"field1":"foo","field2":null,"field3":null}"#
    );

    // Test with no fields
    let subject = TestSubject::new();
    assert_eq!(
        subject.to_json(),
        r#"{"field1":null,"field2":null,"field3":null}"#
    );
}

#[test]
fn subject_derive_json_roundtrip() {
    // Create a subject with mixed values and nulls
    let original = TestSubject {
        field1: Some("test".to_string()),
        field2: None,
        field3: Some("value".to_string()),
    };

    // Convert to JSON string
    let json_str = original.to_json();

    // Convert back from JSON string
    let roundtrip = TestSubject::from_json(&json_str).unwrap();

    // Verify the fields match
    assert_eq!(roundtrip.field1, original.field1);
    assert_eq!(roundtrip.field2, original.field2);
    assert_eq!(roundtrip.field3, original.field3);

    // Verify the parsed subject string is the same
    assert_eq!(roundtrip.parse(), "test.test.*.value");
    assert_eq!(original.parse(), "test.test.*.value");
}
