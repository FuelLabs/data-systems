use fuel_streams_subject::subject::{IndexMap, *};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "test")]
#[subject(entity = "Test")]
#[subject(query_all = "test.>")]
#[subject(format = "test.{field1}.{field2}.{field3}")]
struct TestSubject {
    #[subject(sql_column = "field_id1")]
    pub field1: Option<String>,
    #[subject(sql_column = "field_id2")]
    pub field2: Option<u32>,
    #[subject(sql_column = "field_id3")]
    pub field3: Option<String>,
}

#[test]
fn subject_derive_parse() {
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: Some(55),
        field3: Some("bar".to_string()),
    };

    assert_eq!(TestSubject::QUERY_ALL, "test.>");
    assert_eq!(subject.parse(), "test.foo.55.bar");
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
        Some(
            "field_id1 = 'foo' AND field_id2 = '55' AND field_id3 = 'bar'"
                .to_string()
        )
    );
}

#[test]
fn subject_derive_sql_where_subject_string() {
    let subject = TestSubject {
        field1: None,
        field2: Some(55),
        field3: Some("bar".to_string()),
    };

    assert_eq!(subject.parse(), "test.*.55.bar");
    assert_eq!(
        subject.to_sql_where(),
        Some("field_id2 = '55' AND field_id3 = 'bar'".to_string())
    );
}

#[test]
fn subject_derive_sql_where_greater_than() {
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: None,
        field3: Some("bar".to_string()),
    };

    assert_eq!(
        subject.to_sql_where(),
        Some("field_id1 = 'foo' AND field_id3 = 'bar'".to_string())
    );
}

#[test]
fn subject_derive_sql_where_table_only() {
    let subject = TestSubject {
        field1: None,
        field2: None,
        field3: None,
    };

    assert_eq!(subject.parse(), "test.>");
    assert_eq!(subject.to_sql_where(), None);

    let subject2 = TestSubject::default();
    assert_eq!(subject2.parse(), "test.>");
    assert_eq!(subject2.to_sql_where(), None);

    let subject3 = TestSubject::new();
    assert_eq!(subject3.parse(), "test.>");
    assert_eq!(subject3.to_sql_where(), None);
}

#[test]
fn subject_derive_id() {
    let subject = TestSubject::new();
    assert_eq!(TestSubject::ID, "test");
    assert_eq!(subject.id(), "test");
}

#[test]
fn subject_derive_to_payload() {
    // Test with all fields
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: Some(55),
        field3: Some("bar".to_string()),
    };
    assert_eq!(subject.to_payload(), SubjectPayload {
        subject: subject.id().to_string(),
        params: json!({"field1":"foo","field2":55,"field3":"bar"})
    });

    // Test with no fields
    let subject = TestSubject::new();
    assert_eq!(subject.to_payload(), SubjectPayload {
        subject: subject.id().to_string(),
        params: json!({"field1":null,"field2":null,"field3":null})
    });
}

#[test]
fn subject_derive_roundtrip() {
    // Create original subject with all fields
    let original_subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: Some(55),
        field3: Some("bar".to_string()),
    };

    // Convert to payload
    let payload = original_subject.to_payload();

    // Convert back to subject
    let reconstructed_subject = TestSubject::from(payload);

    // Verify the roundtrip conversion preserved all data
    assert_eq!(original_subject.field1, reconstructed_subject.field1);
    assert_eq!(original_subject.field2, reconstructed_subject.field2);
    assert_eq!(original_subject.field3, reconstructed_subject.field3);

    // Test with empty subject
    let original_empty_subject = TestSubject::new();
    let empty_payload = original_empty_subject.to_payload();
    let reconstructed_empty_subject = TestSubject::from(empty_payload);

    assert_eq!(
        original_empty_subject.field1,
        reconstructed_empty_subject.field1
    );
    assert_eq!(
        original_empty_subject.field2,
        reconstructed_empty_subject.field2
    );
    assert_eq!(
        original_empty_subject.field3,
        reconstructed_empty_subject.field3
    );
}

#[test]
fn subject_derive_entity() {
    let subject = TestSubject::new();
    assert_eq!(TestSubject::ENTITY, "Test");
    assert_eq!(subject.entity(), "Test");
}

#[test]
fn subject_derive_schema() {
    let subject = TestSubject::new();
    let schema = subject.schema();

    let mut fields = IndexMap::new();
    fields.insert("field1".to_string(), FieldSchema {
        type_name: "String".to_string(),
        description: None,
    });
    fields.insert("field2".to_string(), FieldSchema {
        type_name: "u32".to_string(),
        description: None,
    });
    fields.insert("field3".to_string(), FieldSchema {
        type_name: "String".to_string(),
        description: None,
    });

    let expected_schema = Schema {
        id: "test".to_string(),
        entity: "Test".to_string(),
        subject: "TestSubject".to_string(),
        format: "test.{field1}.{field2}.{field3}".to_string(),
        query_all: "test.>".to_string(),
        fields,
        variants: None,
    };

    assert_eq!(schema, expected_schema);
}

#[test]
fn subject_derive_sql_select() {
    // Test with all fields
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: Some(55),
        field3: Some("bar".to_string()),
    };
    assert_eq!(
        subject.to_sql_select(),
        Some("field_id1, field_id2, field_id3".to_string())
    );

    // Test with partial fields
    let subject = TestSubject {
        field1: Some("foo".to_string()),
        field2: None,
        field3: Some("bar".to_string()),
    };
    assert_eq!(
        subject.to_sql_select(),
        Some("field_id1, field_id3".to_string())
    );

    // Test with no fields
    let subject = TestSubject::default();
    assert_eq!(subject.to_sql_select(), None);
}

#[test]
fn subject_derive_sql_where_with_custom_where() {
    #[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
    #[subject(id = "test")]
    #[subject(entity = "Test")]
    #[subject(query_all = "test.>")]
    #[subject(format = "test.{field1}.{field2}.{field3}")]
    #[subject(custom_where = "deleted_at IS NULL")]
    struct TestSubjectWithExtra {
        #[subject(sql_column = "field_id1")]
        pub field1: Option<String>,
        #[subject(sql_column = "field_id2")]
        pub field2: Option<u32>,
        #[subject(sql_column = "field_id3")]
        pub field3: Option<String>,
    }

    // Test with all fields
    let subject = TestSubjectWithExtra {
        field1: Some("foo".to_string()),
        field2: Some(55),
        field3: Some("bar".to_string()),
    };
    assert_eq!(
        subject.to_sql_where(),
        Some("field_id1 = 'foo' AND field_id2 = '55' AND field_id3 = 'bar' AND deleted_at IS NULL".to_string())
    );

    // Test with partial fields
    let subject = TestSubjectWithExtra {
        field1: Some("foo".to_string()),
        field2: None,
        field3: None,
    };
    assert_eq!(
        subject.to_sql_where(),
        Some("field_id1 = 'foo' AND deleted_at IS NULL".to_string())
    );

    // Test with no fields
    let subject = TestSubjectWithExtra::default();
    assert_eq!(
        subject.to_sql_where(),
        Some("deleted_at IS NULL".to_string())
    );
}

#[test]
fn subject_derive_schema_with_descriptions() {
    #[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
    #[subject(id = "test")]
    #[subject(entity = "Test")]
    #[subject(query_all = "test.>")]
    #[subject(format = "test.{field1}.{field2}.{field3}")]
    struct TestSubjectWithDesc {
        #[subject(sql_column = "field_id1")]
        #[subject(description = "The first field description")]
        pub field1: Option<String>,
        #[subject(sql_column = "field_id2", description = "A numeric field")]
        pub field2: Option<u32>,
        #[subject(sql_column = "field_id3", description = "The last field")]
        pub field3: Option<String>,
    }

    let subject = TestSubjectWithDesc::new();
    let schema = subject.schema();

    let mut fields = IndexMap::new();
    fields.insert("field1".to_string(), FieldSchema {
        type_name: "String".to_string(),
        description: Some("The first field description".to_string()),
    });
    fields.insert("field2".to_string(), FieldSchema {
        type_name: "u32".to_string(),
        description: Some("A numeric field".to_string()),
    });
    fields.insert("field3".to_string(), FieldSchema {
        type_name: "String".to_string(),
        description: Some("The last field".to_string()),
    });

    let expected_schema = Schema {
        id: "test".to_string(),
        entity: "Test".to_string(),
        subject: "TestSubjectWithDesc".to_string(),
        format: "test.{field1}.{field2}.{field3}".to_string(),
        query_all: "test.>".to_string(),
        fields,
        variants: None,
    };

    assert_eq!(schema, expected_schema);
}
