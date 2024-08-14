use fuel_streams_macros::subject::*;

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "test.>"]
#[subject_format = "test.{field1}.{field2}"]
struct TestSubject {
    field1: Option<String>,
    field2: Option<u32>,
}

#[test]
fn subject_derive_parse() {
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: Some(55),
    };

    assert_eq!(TestSubject::WILDCARD, "test.>");
    assert_eq!(subject.parse(), "test.foo.55");
}

#[test]
fn subject_derive_wildcard() {
    let wildcard = TestSubject::wildcard(None, Some(10));
    assert_eq!(wildcard, "test.*.10");
}

#[test]
fn subject_derive_build() {
    let subject = TestSubject::build(Some("foo".into()), Some(55));
    assert_eq!(subject.parse(), "test.foo.55");
}

#[test]
fn subject_derive_builder() {
    let subject = TestSubject::new()
        .with_field1(Some("foo".into()))
        .with_field2(Some(55));
    assert_eq!(subject.parse(), "test.foo.55");
}

#[test]
fn subject_derive_to_string() {
    let subject = TestSubject::new()
        .with_field1(Some("foo".into()))
        .with_field2(Some(55));
    assert_eq!(&subject.to_string(), "test.foo.55")
}
