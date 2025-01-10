use fuel_streams_core::types::Block;
use fuel_streams_store::record::Record;
use fuel_streams_test::{
    create_multiple_test_data,
    create_random_db_name,
    setup_store,
};
use futures::StreamExt;

#[tokio::test]
async fn test_stream_by_subject() -> anyhow::Result<()> {
    // Setup store and test data
    let prefix = create_random_db_name();
    let mut store = setup_store::<Block>().await?;
    store.with_namespace(&prefix);
    let data = create_multiple_test_data(10, 0);

    // Insert test records
    for (subject, block) in &data {
        let packet = block
            .to_packet(subject.clone().dyn_arc())
            .with_namespace(&prefix);
        store.insert_record(&packet).await?;
    }

    // Test streaming with the first subject
    let subject = data[0].0.clone();
    let mut stream = store.stream_by_subject(subject.arc(), Some(0));
    let mut count = 0;
    while let Some(result) = stream.next().await {
        let record = result?;
        let height: u32 = data[count].1.height.clone().into();
        assert_eq!(record.block_height as u32, height);
        count += 1;
    }

    // Verify we got all records for this subject
    assert_eq!(count, 1); // We should only get one record since we're querying by specific subject

    Ok(())
}
