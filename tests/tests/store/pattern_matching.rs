use std::sync::Arc;

use fuel_streams_core::{subjects::*, types::Block};
use fuel_streams_domains::blocks::{subjects::BlocksSubject, types::MockBlock};
use fuel_streams_store::record::{QueryOptions, Record};
use fuel_streams_test::{create_random_db_name, setup_store};

#[tokio::test]
async fn test_asterisk_wildcards() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Block>().await?;
    store.with_namespace(&prefix);

    // Create and insert test blocks with different subjects
    let blocks = vec![
        (
            MockBlock::build(1),
            BlocksSubject::new().with_block_height(Some(1.into())),
        ),
        (
            MockBlock::build(2),
            BlocksSubject::new().with_block_height(Some(2.into())),
        ),
        (
            MockBlock::build(3),
            BlocksSubject::new().with_block_height(Some(3.into())),
        ),
    ];

    for (block, subject) in blocks {
        let packet = block.to_packet(Arc::new(subject)).with_namespace(&prefix);
        store.insert_record(&packet).await?;
    }

    // Test wildcard matching
    let subject = BlocksSubject::new().with_block_height(None).dyn_arc();
    let records = store
        .find_many_by_subject(&subject, QueryOptions::default())
        .await?;
    assert_eq!(records.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_nonexistent_subjects() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Block>().await?;
    store.with_namespace(&prefix);

    // Test finding with a subject that doesn't exist
    let nonexistent_subject = BlocksSubject::new()
        .with_block_height(Some(999.into()))
        .dyn_arc();
    let records = store
        .find_many_by_subject(&nonexistent_subject, QueryOptions::default())
        .await?;
    assert!(records.is_empty());

    Ok(())
}
