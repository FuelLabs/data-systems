use std::sync::Arc;

use fuel_streams_core::{subjects::*, types::Block};
use fuel_streams_domains::blocks::{subjects::BlocksSubject, types::MockBlock};
use fuel_streams_store::record::Record;
use fuel_streams_test::{create_random_db_name, setup_store};

#[tokio::test]
async fn test_asterisk_wildcards() -> anyhow::Result<()> {
    let store = setup_store().await?;
    let prefix = create_random_db_name();

    // Create and insert test blocks with different subjects
    let blocks = vec![
        (
            MockBlock::build(1),
            BlocksSubject::new().with_height(Some(1.into())),
        ),
        (
            MockBlock::build(2),
            BlocksSubject::new().with_height(Some(2.into())),
        ),
        (
            MockBlock::build(3),
            BlocksSubject::new().with_height(Some(3.into())),
        ),
    ];

    for (block, subject) in blocks {
        let packet = block.to_packet(Arc::new(subject)).with_namespace(&prefix);
        store.insert_record(&packet).await?;
    }

    // Test wildcard matching
    let wildcard_subject = BlocksSubject::new().with_height(None).dyn_arc();
    let records = store
        .find_many_by_subject_ns(&wildcard_subject, &prefix, 0, 10, None)
        .await?;
    assert_eq!(records.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_nonexistent_subjects() -> anyhow::Result<()> {
    let store = setup_store::<Block>().await?;
    let prefix = create_random_db_name();

    // Test finding with a subject that doesn't exist
    let nonexistent_subject =
        BlocksSubject::new().with_height(Some(999.into())).dyn_arc();
    let records = store
        .find_many_by_subject_ns(&nonexistent_subject, &prefix, 0, 10, None)
        .await?;
    assert!(records.is_empty());

    Ok(())
}
