use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use fuel_core::combined_database::CombinedDatabase;
use fuel_core_importer::ImporterResult;
use fuel_core_types::{
    blockchain::SealedBlock,
    fuel_tx::{Address, AssetId, Bytes32, ContractId},
};
use fuel_streams_core::{
    blocks::BlocksSubject,
    nats::{NatsClient, NatsClientOpts},
    prelude::*,
    types::ImportResult,
};
use fuel_streams_publisher::{FuelCoreLike, Publisher};
use futures::StreamExt;
use tokio::sync::{broadcast, broadcast::Receiver};

struct TestFuelCore {
    chain_id: ChainId,
    database: CombinedDatabase,
    blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    receipts: Option<Vec<Receipt>>,
}

impl TestFuelCore {
    fn default(
        blocks_subscription: Receiver<fuel_core_importer::ImporterResult>,
    ) -> Self {
        Self {
            chain_id: ChainId::default(),
            database: CombinedDatabase::default(),
            blocks_subscription,
            receipts: None,
        }
    }
    fn with_receipts(mut self, receipts: Vec<Receipt>) -> Self {
        self.receipts = Some(receipts);
        self
    }
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

#[async_trait::async_trait]
impl FuelCoreLike for TestFuelCore {
    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    fn database(&self) -> &CombinedDatabase {
        &self.database
    }

    fn blocks_subscription(
        &mut self,
    ) -> &mut Receiver<fuel_core_importer::ImporterResult> {
        &mut self.blocks_subscription
    }

    fn get_receipts(
        &self,
        _tx_id: &Bytes32,
    ) -> anyhow::Result<Option<Vec<Receipt>>> {
        Ok(self.receipts.clone())
    }
}

#[tokio::test]
async fn doesnt_publish_any_message_when_no_block_has_been_mined() {
    let (_, blocks_subscription) = broadcast::channel::<ImporterResult>(1);
    let fuel_core = TestFuelCore::default(blocks_subscription).boxed();

    let publisher = Publisher::new(&nats_client().await, fuel_core).await;
    let publisher = publisher.run().await.unwrap();

    assert!(publisher.get_streams().is_empty().await);
}

#[tokio::test]
async fn publishes_a_block_message_when_a_single_block_has_been_mined() {
    let (blocks_subscriber, blocks_subscription) =
        broadcast::channel::<ImporterResult>(1);

    let block = ImporterResult {
        shared_result: Arc::new(ImportResult::default()),
        changes: Arc::new(HashMap::new()),
    };
    let _ = blocks_subscriber.send(block);

    // manually drop blocks to ensure `blocks_subscription` completes
    let _ = blocks_subscriber.clone();
    drop(blocks_subscriber);

    let fuel_core = TestFuelCore::default(blocks_subscription).boxed();
    let publisher = Publisher::new(&nats_client().await, fuel_core).await;
    let publisher = publisher.run().await.unwrap();

    assert!(publisher
        .get_streams()
        .blocks
        .get_last_published(BlocksSubject::WILDCARD)
        .await
        .is_ok_and(|result| result.is_some()));
}

#[tokio::test]
async fn publishes_transaction_for_each_published_block() {
    let (blocks_subscriber, blocks_subscription) =
        broadcast::channel::<ImporterResult>(1);

    let mut block_entity = Block::default();
    *block_entity.transactions_mut() = vec![Transaction::default_test_tx()];

    // publish block
    let block = ImporterResult {
        shared_result: Arc::new(ImportResult {
            sealed_block: SealedBlock {
                entity: block_entity,
                ..Default::default()
            },
            ..Default::default()
        }),
        changes: Arc::new(HashMap::new()),
    };
    let _ = blocks_subscriber.send(block);

    // manually drop blocks to ensure `blocks_subscription` completes
    let _ = blocks_subscriber.clone();
    drop(blocks_subscriber);

    let fuel_core = TestFuelCore::default(blocks_subscription).boxed();
    let publisher = Publisher::new(&nats_client().await, fuel_core).await;
    let publisher = publisher.run().await.unwrap();

    assert!(publisher
        .get_streams()
        .transactions
        .get_last_published(TransactionsSubject::WILDCARD)
        .await
        .is_ok_and(|result| result.is_some()));
}

#[tokio::test]
async fn publishes_receipts() {
    let (blocks_subscriber, blocks_subscription) =
        broadcast::channel::<ImporterResult>(1);

    let mut block_entity = Block::default();
    *block_entity.transactions_mut() = vec![Transaction::default_test_tx()];

    // publish block
    let block = ImporterResult {
        shared_result: Arc::new(ImportResult {
            sealed_block: SealedBlock {
                entity: block_entity,
                ..Default::default()
            },
            ..Default::default()
        }),
        changes: Arc::new(HashMap::new()),
    };
    let _ = blocks_subscriber.send(block);

    let _ = blocks_subscriber.clone();
    drop(blocks_subscriber);

    let receipts = [
        Receipt::Call {
            id: ContractId::default(),
            to: Default::default(),
            amount: 0,
            asset_id: Default::default(),
            gas: 0,
            param1: 0,
            param2: 0,
            pc: 0,
            is: 0,
        },
        Receipt::Return {
            id: ContractId::default(),
            val: 0,
            pc: 0,
            is: 0,
        },
        Receipt::ReturnData {
            id: ContractId::default(),
            ptr: 0,
            len: 0,
            digest: Bytes32::default(),
            pc: 0,
            is: 0,
            data: None,
        },
        Receipt::Revert {
            id: ContractId::default(),
            ra: 0,
            pc: 0,
            is: 0,
        },
        Receipt::Log {
            id: ContractId::default(),
            ra: 0,
            rb: 0,
            rc: 0,
            rd: 0,
            pc: 0,
            is: 0,
        },
        Receipt::LogData {
            id: ContractId::default(),
            ra: 0,
            rb: 0,
            ptr: 0,
            len: 0,
            digest: Bytes32::default(),
            pc: 0,
            is: 0,
            data: None,
        },
        Receipt::Transfer {
            id: ContractId::default(),
            to: ContractId::default(),
            amount: 0,
            asset_id: AssetId::default(),
            pc: 0,
            is: 0,
        },
        Receipt::TransferOut {
            id: ContractId::default(),
            to: Address::default(),
            amount: 0,
            asset_id: AssetId::default(),
            pc: 0,
            is: 0,
        },
        Receipt::Mint {
            sub_id: Bytes32::default(),
            contract_id: ContractId::default(),
            val: 0,
            pc: 0,
            is: 0,
        },
        Receipt::Burn {
            sub_id: Bytes32::default(),
            contract_id: ContractId::default(),
            val: 0,
            pc: 0,
            is: 0,
        },
    ];

    let fuel_core = TestFuelCore::default(blocks_subscription)
        .with_receipts(receipts.to_vec())
        .boxed();

    let publisher = Publisher::new(&nats_client().await, fuel_core).await;

    let publisher = publisher.run().await.unwrap();

    let mut receipts_stream =
        publisher.get_streams().receipts.catchup(10).await.unwrap();

    let receipts: HashSet<Receipt> = receipts.into();
    while let Some(Some(receipt)) = receipts_stream.next().await {
        assert!(receipts.contains(&receipt));
    }
}

async fn nats_client() -> NatsClient {
    const NATS_URL: &str = "nats://localhost:4222";
    let nats_client_opts =
        NatsClientOpts::admin_opts(NATS_URL).with_rdn_namespace();
    NatsClient::connect(&nats_client_opts)
        .await
        .expect("NATS connection failed")
}
