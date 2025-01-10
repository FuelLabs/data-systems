use std::sync::Arc;

use fuel_streams_core::{subjects::*, types::Block};
use fuel_streams_domains::blocks::{
    subjects::BlocksSubject,
    types::MockBlock,
    BlockDbItem,
};
use fuel_streams_store::record::{Record, RecordPacket};
use fuel_streams_test::{create_random_db_name, setup_store};

#[tokio::test]
async fn test_block_db_item_conversion() -> anyhow::Result<()> {
    let block = MockBlock::build(1);
    let subject = BlocksSubject::from(&block);

    // Create Arc<BlocksSubject> explicitly and use RecordPacket::new
    let subject_arc: Arc<BlocksSubject> = Arc::new(subject.clone());
    let packet = RecordPacket::new(subject_arc, &block);

    // Test direct conversion
    let db_item = BlockDbItem::try_from(&packet)
        .expect("Failed to convert packet to BlockDbItem");

    let height: i64 = block.height.clone().into();
    assert_eq!(db_item.subject, subject.parse());
    assert_eq!(db_item.height, height);
    assert_eq!(db_item.producer_address, block.producer.to_string());

    // Verify we can decode the value back to a block
    let decoded_block: Block = serde_json::from_slice(&db_item.value)
        .expect("Failed to decode block from value");
    assert_eq!(decoded_block, block);
    Ok(())
}

#[tokio::test]
async fn test_basic_insert() -> anyhow::Result<()> {
    let store = setup_store().await?;
    let block = MockBlock::build(1);
    let subject = BlocksSubject::from(&block);
    let prefix = create_random_db_name();
    let packet = block
        .to_packet(Arc::new(subject.clone()))
        .with_namespace(&prefix);

    let db_record = store.insert_record(&packet).await?;
    assert_eq!(db_record.subject, packet.subject_str());
    assert_eq!(Block::from_db_item(&db_record).await?, block);

    Ok(())
}

#[tokio::test]
async fn test_multiple_inserts() -> anyhow::Result<()> {
    let store = setup_store().await?;
    let subject = BlocksSubject::from(&MockBlock::build(1));

    // Insert first block
    let block1 = MockBlock::build(1);
    let prefix = create_random_db_name();
    let packet = block1
        .to_packet(Arc::new(subject.clone()))
        .with_namespace(&prefix);
    let db_record1 = store.insert_record(&packet).await?;

    // Insert second block
    let block2 = MockBlock::build(2);
    let packet = block2
        .to_packet(Arc::new(subject.clone()))
        .with_namespace(&prefix);
    let db_record2 = store.insert_record(&packet).await?;

    // Verify both records exist and are correct
    assert_eq!(Block::from_db_item(&db_record1).await?, block1);
    assert_eq!(Block::from_db_item(&db_record2).await?, block2);

    // Verify both records are found
    let subject = BlocksSubject::new().with_height(None).dyn_arc();
    let records = store
        .find_many_by_subject_ns(&subject, &prefix, 0, 10, None)
        .await?;
    assert_eq!(records.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_find_many_by_subject() -> anyhow::Result<()> {
    let store = setup_store().await?;
    let prefix = create_random_db_name();
    let subject1 = BlocksSubject::from(&MockBlock::build(1));
    let subject2 = BlocksSubject::from(&MockBlock::build(2));

    // Insert blocks with different subjects
    let block1 = MockBlock::build(1);
    let block2 = MockBlock::build(2);
    let packet1 = block1
        .to_packet(Arc::new(subject1.clone()))
        .with_namespace(&prefix);
    let packet2 = block2
        .to_packet(Arc::new(subject2.clone()))
        .with_namespace(&prefix);

    store.insert_record(&packet1).await?;
    store.insert_record(&packet2).await?;

    // Test finding by subject1
    let records = store
        .find_many_by_subject_ns(&subject1.dyn_arc(), &prefix, 0, 10, None)
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(Block::from_db_item(&records[0]).await?, block1);

    // Test finding by subject2
    let records = store
        .find_many_by_subject_ns(&subject2.dyn_arc(), &prefix, 0, 10, None)
        .await?;
    assert_eq!(records.len(), 1);
    assert_eq!(Block::from_db_item(&records[0]).await?, block2);

    Ok(())
}

#[tokio::test]
async fn test_find_last_record() -> anyhow::Result<()> {
    let store = setup_store().await?;
    let subject = BlocksSubject::from(&MockBlock::build(1));

    // Insert multiple blocks
    let blocks = vec![
        MockBlock::build(1),
        MockBlock::build(2),
        MockBlock::build(3),
    ];

    for block in &blocks {
        store
            .insert_record(&block.to_packet(Arc::new(subject.clone())))
            .await?;
    }

    // Test finding last record
    let last_record = Block::find_last_record(&store.db).await?;
    assert!(last_record.is_some());
    let last_block = Block::from_db_item(&last_record.unwrap()).await?;
    assert_eq!(last_block, blocks.last().unwrap().clone());

    Ok(())
}

#[tokio::test]
async fn test_subject_matching() -> anyhow::Result<()> {
    let block = MockBlock::build(1);
    let subject = BlocksSubject::from(&block);
    let packet = block.to_packet(Arc::new(subject.clone()));

    // Test subject matching
    let matched_subject: BlocksSubject = packet
        .subject_matches()
        .expect("Failed to match BlocksSubject");

    assert_eq!(matched_subject.parse(), subject.parse());

    Ok(())
}
