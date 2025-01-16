use fuel_streams_core::{subjects::*, types::Block};
use fuel_streams_domains::blocks::{subjects::BlocksSubject, types::MockBlock};
use fuel_streams_store::record::{QueryOptions, Record};
use fuel_streams_test::{
    create_multiple_records,
    create_random_db_name,
    insert_records,
    insert_records_with_transaction,
    setup_store,
};
use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_multiple_inserts() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Block>().await?;
    store.with_namespace(&prefix);

    let blocks = create_multiple_records(2, 1);
    let db_items = insert_records(&store, &prefix, &blocks).await?;

    // Verify both records exist and are correct
    let db_record1 = db_items.first().unwrap();
    let db_record2 = db_items.get(1).unwrap();
    let block1 = &blocks.first().unwrap().1;
    let block2 = &blocks.get(1).unwrap().1;
    assert_eq!(&Block::from_db_item(db_record1).await?, block1);
    assert_eq!(&Block::from_db_item(db_record2).await?, block2);

    // Verify both records are found
    let subject = BlocksSubject::new().with_height(None).dyn_arc();
    let records = store
        .find_many_by_subject(&subject, QueryOptions::default())
        .await?;
    assert_eq!(records.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_find_many_by_subject() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Block>().await?;
    store.with_namespace(&prefix);

    let blocks = create_multiple_records(2, 1);
    let _ = insert_records(&store, &prefix, &blocks).await?;
    let block1 = &blocks.first().unwrap().1;
    let block2 = &blocks.get(1).unwrap().1;

    // Test finding by subject1
    let subject1 = BlocksSubject::build(None, Some(1.into())).dyn_arc();
    let records = store
        .find_many_by_subject(&subject1, QueryOptions::default())
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(&Block::from_db_item(&records[0]).await?, block1);

    // Test finding by subject2
    let subject2 = BlocksSubject::build(None, Some(2.into())).dyn_arc();
    let records = store
        .find_many_by_subject(&subject2, QueryOptions::default())
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(&Block::from_db_item(&records[0]).await?, block2);

    Ok(())
}

#[tokio::test]
async fn test_find_last_record() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Block>().await?;
    store.with_namespace(&prefix);

    // Insert multiple blocks
    let blocks = create_multiple_records(4, 1);
    let _ = insert_records(&store, &prefix, &blocks).await?;
    let block4 = &blocks.get(3).unwrap().1;

    // Test finding last record
    let last_record = store.find_last_record().await?;
    assert!(last_record.is_some());
    let last_block = Block::from_db_item(&last_record.unwrap()).await?;
    assert_eq!(&last_block, block4);

    Ok(())
}

#[tokio::test]
async fn test_subject_matching() -> anyhow::Result<()> {
    let block = MockBlock::build(1);
    let subject = BlocksSubject::from(&block).dyn_arc();
    let packet = block.to_packet(&subject);

    // Test subject matching
    let matched_subject: BlocksSubject = packet
        .subject_matches()
        .expect("Failed to match BlocksSubject");
    assert_eq!(matched_subject.parse(), subject.parse());
    Ok(())
}

#[tokio::test]
async fn test_insert_with_transaction() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Block>().await?;
    store.with_namespace(&prefix);

    // Start a transaction
    let mut tx = store.db.pool.begin().await?;
    let blocks = create_multiple_records(4, 1);
    insert_records_with_transaction(&store, &mut tx, &prefix, &blocks).await?;
    tx.commit().await?;

    // Verify all records were inserted
    let subject = BlocksSubject::new().with_height(None).dyn_arc();
    let found_records = store
        .find_many_by_subject(&subject, QueryOptions::default())
        .await?;
    assert_eq!(found_records.len(), 4);

    // Verify the records match the original blocks
    for (record, item) in found_records.iter().zip(blocks.iter()) {
        let (_, block) = item;
        assert_eq!(&Block::from_db_item(record).await?, block);
    }

    Ok(())
}
