use fuel_streams_core::{subjects::*, types::Block};
use fuel_streams_domains::{
    blocks::{subjects::BlocksSubject, types::MockBlock, BlockDbItem},
    MockMsgPayload,
};
use fuel_streams_store::record::Record;
use fuel_streams_test::{close_db, create_random_db_name, setup_store};
use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_block_db_item_conversion() -> anyhow::Result<()> {
    let block = MockBlock::build(1);
    let subject = BlocksSubject::from(&block).dyn_arc();
    let msg_payload = MockMsgPayload::from(&block).into_inner();
    let timestamps = msg_payload.timestamp();
    let packet = block.to_packet(&subject, timestamps);

    // Test direct conversion
    let db_item = BlockDbItem::try_from(&packet)
        .expect("Failed to convert packet to BlockDbItem");

    let height: i64 = block.height.into();
    let da_height: i64 = block.header.da_height.into();
    assert_eq!(db_item.subject, subject.parse());
    assert_eq!(db_item.block_da_height, da_height.into());
    assert_eq!(db_item.block_height, height.into());
    assert_eq!(db_item.producer_address, block.producer.to_string());

    // Verify we can decode the value back to a block
    let decoded_block: Block = serde_json::from_slice(&db_item.value)
        .expect("Failed to decode block from value");
    assert_eq!(decoded_block, block);
    Ok(())
}

#[tokio::test]
async fn store_can_record_blocks() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let store = setup_store::<Block>().await?;
    let block = MockBlock::build(1);
    let subject = BlocksSubject::from(&block).dyn_arc();
    let msg_payload = MockMsgPayload::build(1, &prefix);
    let timestamps = msg_payload.timestamp();
    let packet = block.to_packet(&subject, timestamps);
    let packet = packet.with_namespace(&prefix);
    let db_item = BlockDbItem::try_from(&packet)?;
    let inserted = store.insert_record(&db_item).await?;
    assert_eq!(inserted.block_da_height, db_item.block_da_height);
    assert_eq!(inserted.block_height, db_item.block_height);
    assert_eq!(inserted.producer_address, db_item.producer_address);
    assert_eq!(inserted.subject, db_item.subject);
    assert_eq!(inserted.value, db_item.value);
    assert_eq!(inserted.created_at, db_item.created_at);
    assert_eq!(inserted.subject, packet.subject_str());
    assert!(inserted.published_at.is_after(&db_item.created_at));
    assert_eq!(Block::from_db_item(&inserted)?, block);

    close_db(&store.db).await;
    Ok(())
}
