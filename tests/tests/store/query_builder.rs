use std::sync::Arc;

use fuel_streams_core::{
    inputs::InputsCoinSubject,
    subjects::*,
    types::{Block, Input},
};
use fuel_streams_domains::blocks::subjects::BlocksSubject;
use fuel_streams_store::record::{QueryOptions, Record};
use fuel_streams_types::{Address, TxId};
use pretty_assertions::assert_eq;

#[test]
fn test_query_builder_simple() {
    let subject = Arc::new(
        BlocksSubject::new()
            .with_height(Some(50.into()))
            .with_producer(Some(Address::default())),
    );

    let options = QueryOptions {
        offset: 0,
        limit: 10,
        from_block: None,
        namespace: Some("test_ns".to_string()),
    };

    let sql_statement = subject.to_sql_select();
    let sql_where = subject.to_sql_where();
    assert_eq!(
        sql_statement,
        Some("producer_address, block_height".to_string())
    );
    assert_eq!(
        sql_where,
        Some(format!(
            "producer_address = '{}' AND block_height = '50'",
            Address::default()
        ))
    );

    let query = Block::build_find_many_query(subject, options);
    let sql = query.sql();

    assert_eq!(
        sql,
        format!(
            "SELECT * FROM blocks \
            WHERE producer_address = '{}' AND block_height = '50' \
            AND subject LIKE 'test_ns%' \
            ORDER BY block_height ASC \
            LIMIT $1 OFFSET $2",
            Address::default()
        )
    );
}

#[test]
fn test_query_builder_with_no_subject_fields() {
    let subject = Arc::new(BlocksSubject::new());
    let options = QueryOptions::default();
    let query = Block::build_find_many_query(subject, options);
    let sql = query.sql();

    assert_eq!(
        sql,
        "SELECT * FROM blocks \
        ORDER BY block_height ASC \
        LIMIT $1 OFFSET $2"
    );
}

#[test]
fn test_query_builder_with_complex_ordering() {
    let tx_id = TxId::default();
    let subject = Arc::new(InputsCoinSubject {
        block_height: None,
        tx_id: Some(tx_id),
        tx_index: None,
        input_index: None,
        owner: Some(Address::default()),
        asset: None,
    });

    let options = QueryOptions {
        offset: 0,
        limit: 20,
        from_block: Some(50),
        namespace: Some("test_ns".to_string()),
    };

    let query = Input::build_find_many_query(subject, options);
    let sql = query.sql();

    assert_eq!(
        sql,
        format!(
            "WITH items AS (\
            SELECT * FROM inputs \
            WHERE tx_id = '{}' \
            AND owner_id = '{}' \
            AND input_type = 'coin' \
            AND block_height >= 50 \
            AND subject LIKE 'test_ns%' \
            ORDER BY block_height ASC \
            LIMIT $1 OFFSET $2) \
            SELECT * FROM items \
            ORDER BY tx_index, input_index ASC",
            TxId::default(),
            Address::default(),
        )
    );
}
