use fuel_streams_core::{
    subjects::{SubjectBuildable, TransactionsSubject},
    types::{MockInput, MockOutput, MockReceipt, MockTransaction, Transaction},
};
use fuel_streams_domains::{transactions::TransactionDbItem, Subjects};
use fuel_streams_store::{
    record::{QueryOptions, Record, RecordPacket},
    store::Store,
};
use fuel_streams_test::{create_random_db_name, setup_db, setup_store};
use pretty_assertions::assert_eq;

async fn insert_transaction(tx: &Transaction) -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let packets = create_packets(tx, &prefix);
    assert_eq!(packets.len(), 1);

    let mut store = setup_store::<Transaction>().await?;
    let packet = packets.first().unwrap().clone();
    store.with_namespace(&prefix);

    let db_item = TransactionDbItem::try_from(&packet);
    assert!(
        db_item.is_ok(),
        "Failed to convert packet to db item: {:?}",
        packet
    );

    let db_item = db_item.unwrap();
    let db_record = store.insert_record(&db_item).await?;
    assert_eq!(db_record.subject, packet.subject_str());

    Ok(())
}

fn create_packets(tx: &Transaction, prefix: &str) -> Vec<RecordPacket> {
    let subject = TransactionsSubject::new()
        .with_block_height(Some(1.into()))
        .with_tx_id(Some(tx.id.clone()))
        .with_tx_index(Some(0))
        .with_tx_status(Some(tx.status.clone()))
        .with_kind(Some(tx.kind.clone()))
        .dyn_arc();
    vec![tx.to_packet(&subject).with_namespace(prefix)]
}

#[tokio::test]
async fn test_store_script_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::script(
        vec![MockInput::coin_signed()],
        vec![MockOutput::coin(100)],
        vec![MockReceipt::script_result()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_create_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::create(
        vec![MockInput::contract()],
        vec![MockOutput::contract()],
        vec![MockReceipt::call()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_mint_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::mint(
        vec![MockInput::contract()],
        vec![MockOutput::coin(1000)],
        vec![MockReceipt::mint()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_upgrade_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::upgrade(
        vec![MockInput::coin_signed()],
        vec![MockOutput::coin(100)],
        vec![MockReceipt::script_result()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_upload_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::upload(
        vec![MockInput::coin_signed()],
        vec![MockOutput::coin(100)],
        vec![MockReceipt::script_result()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn test_store_blob_transaction() -> anyhow::Result<()> {
    let tx = MockTransaction::blob(
        vec![MockInput::coin_signed()],
        vec![MockOutput::coin(100)],
        vec![MockReceipt::script_result()],
    );
    insert_transaction(&tx).await
}

#[tokio::test]
async fn find_many_by_subject_with_sql_columns() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let mut store = setup_store::<Transaction>().await?;
    store.with_namespace(&prefix);

    // Create transactions of each type
    let transactions = vec![
        MockTransaction::script(
            vec![MockInput::coin_signed()],
            vec![MockOutput::coin(100)],
            vec![MockReceipt::script_result()],
        ),
        MockTransaction::create(
            vec![MockInput::contract()],
            vec![MockOutput::contract()],
            vec![MockReceipt::call()],
        ),
        MockTransaction::mint(
            vec![MockInput::contract()],
            vec![MockOutput::coin(1000)],
            vec![MockReceipt::mint()],
        ),
        MockTransaction::upgrade(
            vec![MockInput::coin_signed()],
            vec![MockOutput::coin(100)],
            vec![MockReceipt::script_result()],
        ),
        MockTransaction::upload(
            vec![MockInput::coin_signed()],
            vec![MockOutput::coin(100)],
            vec![MockReceipt::script_result()],
        ),
        MockTransaction::blob(
            vec![MockInput::coin_signed()],
            vec![MockOutput::coin(100)],
            vec![MockReceipt::script_result()],
        ),
    ];

    for tx in transactions {
        let packets = create_packets(&tx, &prefix);
        for packet in packets {
            let payload = packet.subject_payload.clone();
            let subject: Subjects = payload.try_into()?;
            let subject = subject.into();
            let _ = store
                .find_many_by_subject(&subject, QueryOptions::default())
                .await?;
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_transaction_subject_to_db_item_conversion() -> anyhow::Result<()>
{
    let prefix = create_random_db_name();
    let db = setup_db().await?;
    let mut store = Store::<Transaction>::new(&db.arc());
    store.with_namespace(&prefix);

    let transactions = vec![
        MockTransaction::script(
            vec![MockInput::coin_signed()],
            vec![MockOutput::coin(100)],
            vec![MockReceipt::script_result()],
        ),
        MockTransaction::create(
            vec![MockInput::contract()],
            vec![MockOutput::contract()],
            vec![MockReceipt::call()],
        ),
        MockTransaction::mint(
            vec![MockInput::contract()],
            vec![MockOutput::coin(1000)],
            vec![MockReceipt::mint()],
        ),
        MockTransaction::upgrade(
            vec![MockInput::coin_signed()],
            vec![MockOutput::coin(100)],
            vec![MockReceipt::script_result()],
        ),
        MockTransaction::upload(
            vec![MockInput::coin_signed()],
            vec![MockOutput::coin(100)],
            vec![MockReceipt::script_result()],
        ),
        MockTransaction::blob(
            vec![MockInput::coin_signed()],
            vec![MockOutput::coin(100)],
            vec![MockReceipt::script_result()],
        ),
    ];

    for tx in transactions {
        let packets = create_packets(&tx, &prefix);
        let packet = packets.first().unwrap();
        let payload = packet.subject_payload.clone();
        let subject: Subjects = payload.try_into()?;
        let db_item = TransactionDbItem::try_from(packet)?;
        let inserted = store.insert_record(&db_item).await?;
        assert_eq!(db_item, inserted);

        // Verify common fields
        assert_eq!(db_item.block_height, 1);
        assert_eq!(db_item.tx_id, tx.id.to_string());
        assert_eq!(db_item.tx_index, 0);
        assert_eq!(db_item.subject, packet.subject_str());

        match subject {
            Subjects::Transactions(subject) => {
                assert_eq!(
                    db_item.tx_status,
                    subject.tx_status.unwrap().to_string()
                );
                assert_eq!(db_item.kind, subject.kind.unwrap().to_string());
            }
            _ => panic!("Unexpected subject type"),
        }
    }

    Ok(())
}
