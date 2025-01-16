use fuel_streams_core::{subjects::*, types::Block};
use fuel_streams_domains::blocks::{
    subjects::BlocksSubject,
    types::MockBlock,
    BlockDbItem,
};
use fuel_streams_store::record::Record;
use fuel_streams_test::{create_random_db_name, setup_store};

#[tokio::test]
async fn test_block_db_item_conversion() -> anyhow::Result<()> {
    let block = MockBlock::build(1);
    let subject = BlocksSubject::from(&block).dyn_arc();
    let packet = block.to_packet(&subject);

    // Test direct conversion
    let db_item = BlockDbItem::try_from(&packet)
        .expect("Failed to convert packet to BlockDbItem");

    let height: i64 = block.height.clone().into();
    assert_eq!(db_item.subject, subject.parse());
    assert_eq!(db_item.block_height, height);
    assert_eq!(db_item.producer_address, block.producer.to_string());

    // Verify we can decode the value back to a block
    let decoded_block: Block = serde_json::from_slice(&db_item.value)
        .expect("Failed to decode block from value");
    assert_eq!(decoded_block, block);
    Ok(())
}

#[tokio::test]
async fn store_can_record_blocks() -> anyhow::Result<()> {
    let store = setup_store::<Block>().await?;
    let block = MockBlock::build(1);
    let subject = BlocksSubject::from(&block).dyn_arc();
    let prefix = create_random_db_name();
    let packet = block.to_packet(&subject).with_namespace(&prefix);

    let db_record: BlockDbItem = store.insert_record(&packet).await?;
    assert_eq!(db_record.subject, packet.subject_str());
    assert_eq!(Block::from_db_item(&db_record).await?, block);

    Ok(())
}
