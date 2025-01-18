use std::sync::Arc;

use fuel_streams_core::{
    inputs::{InputsCoinSubject, InputsContractSubject, InputsMessageSubject},
    subjects::*,
    types::{Block, Input},
};
use fuel_streams_domains::blocks::subjects::BlocksSubject;
use fuel_streams_store::record::{QueryOptions, Record};
use fuel_streams_types::{Address, AssetId, ContractId, TxId};
use pretty_assertions::assert_eq;

#[test]
fn test_query_builder() {
    let subject = Arc::new(
        BlocksSubject::new()
            .with_height(Some(50.into()))
            .with_producer(Some(Address::default())),
    );

    let options = QueryOptions {
        offset: 0,
        limit: 10,
        from_block: Some(100),
        to_block: Some(200),
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
            r#"SELECT _id, subject, value, block_height FROM blocks WHERE producer_address = '{}' AND block_height = '50' AND block_height >= 100 AND block_height < 200 AND subject LIKE 'test_ns%' ORDER BY block_height ASC LIMIT $1 OFFSET $2"#,
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
        r#"SELECT _id, subject, value, block_height FROM blocks ORDER BY block_height ASC LIMIT $1 OFFSET $2"#
    );
}

#[test]
fn test_query_builder_coin_input() {
    let tx_id = TxId::default();
    let subject = Arc::new(InputsCoinSubject {
        block_height: Some(100.into()),
        tx_id: Some(tx_id),
        tx_index: Some(1),
        input_index: Some(2),
        owner: Some(Address::default()),
        asset: Some(AssetId::default()),
    });

    let options = QueryOptions {
        offset: 0,
        limit: 20,
        from_block: Some(50),
        to_block: Some(150),
        namespace: Some("test_ns".to_string()),
    };

    let query = Input::build_find_many_query(subject, options);
    let sql = query.sql();

    assert_eq!(
        sql,
        format!(
            r#"SELECT _id, subject, value, block_height, tx_index, input_index FROM inputs WHERE block_height = '100' AND tx_id = '{}' AND tx_index = '1' AND input_index = '2' AND owner_id = '{}' AND asset_id = '{}' AND block_height >= 50 AND block_height < 150 AND subject LIKE 'test_ns%' ORDER BY block_height, tx_index, input_index ASC LIMIT $1 OFFSET $2"#,
            TxId::default(),
            Address::default(),
            AssetId::default(),
        )
    );
}

#[test]
fn test_query_builder_contract_input() {
    let contract_id = ContractId::default();
    let subject = Arc::new(InputsContractSubject {
        block_height: Some(100.into()),
        tx_id: None,
        tx_index: None,
        input_index: None,
        contract: Some(contract_id.clone()),
    });

    let options = QueryOptions::default();
    let query = Input::build_find_many_query(subject, options);
    let sql = query.sql();

    assert_eq!(
        sql,
        format!(
            r#"SELECT _id, subject, value, block_height, tx_index, input_index FROM inputs WHERE block_height = '100' AND contract_id = '{}' ORDER BY block_height, tx_index, input_index ASC LIMIT $1 OFFSET $2"#,
            contract_id,
        )
    );
}

#[test]
fn test_query_builder_message_input() {
    let sender = Address::default();
    let subject = Arc::new(InputsMessageSubject {
        block_height: None,
        tx_id: None,
        tx_index: None,
        input_index: None,
        sender: Some(sender.clone()),
        recipient: None,
    });

    let options = QueryOptions::default();
    let query = Input::build_find_many_query(subject, options);
    let sql = query.sql();

    assert_eq!(
        sql,
        format!(
            r#"SELECT _id, subject, value, block_height, tx_index, input_index FROM inputs WHERE sender_address = '{}' ORDER BY block_height, tx_index, input_index ASC LIMIT $1 OFFSET $2"#,
            sender,
        )
    );
}

#[test]
fn test_query_builder_empty_subject() {
    let subject = Arc::new(InputsCoinSubject::new());
    let options = QueryOptions::default();

    let query = Input::build_find_many_query(subject, options);
    let sql = query.sql();

    assert_eq!(
        sql,
        r#"SELECT _id, subject, value, block_height, tx_index, input_index FROM inputs ORDER BY block_height, tx_index, input_index ASC LIMIT $1 OFFSET $2"#
    );
}

#[test]
fn test_query_builder_only_block_range() {
    let subject = Arc::new(InputsMessageSubject::new());
    let options = QueryOptions {
        offset: 0,
        limit: 50,
        from_block: Some(100),
        to_block: Some(200),
        namespace: None,
    };

    let query = Input::build_find_many_query(subject, options);
    let sql = query.sql();

    assert_eq!(
        sql,
        r#"SELECT _id, subject, value, block_height, tx_index, input_index FROM inputs WHERE block_height >= 100 AND block_height < 200 ORDER BY block_height, tx_index, input_index ASC LIMIT $1 OFFSET $2"#
    );
}
