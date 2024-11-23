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
use fuel_streams_core::prelude::*;
use fuel_streams_publisher::{
    publisher::shutdown::ShutdownController,
    FuelCoreLike,
    Publisher,
};
use futures::StreamExt;
use tokio::sync::broadcast::{self, Receiver, Sender};

struct TestFuelCore {
    chain_id: ChainId,
    database: CombinedDatabase,
    blocks_broadcaster: Sender<fuel_core_importer::ImporterResult>,
    receipts: Option<Vec<FuelCoreReceipt>>,
}

impl TestFuelCore {
    fn default(
        blocks_broadcaster: Sender<fuel_core_importer::ImporterResult>,
    ) -> Self {
        Self {
            chain_id: ChainId::default(),
            database: CombinedDatabase::default(),
            blocks_broadcaster,
            receipts: None,
        }
    }
    fn with_receipts(mut self, receipts: Vec<FuelCoreReceipt>) -> Self {
        self.receipts = Some(receipts);
        self
    }
    fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[async_trait::async_trait]
impl FuelCoreLike for TestFuelCore {
    async fn start(&self) {}
    fn is_started(&self) -> bool {
        true
    }
    async fn stop(&self) {}

    fn base_asset_id(&self) -> &AssetId {
        unimplemented!()
    }
    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    fn database(&self) -> &CombinedDatabase {
        &self.database
    }

    fn blocks_subscription(
        &self,
    ) -> Receiver<fuel_core_importer::ImporterResult> {
        self.blocks_broadcaster.subscribe()
    }

    fn get_receipts(
        &self,
        _tx_id: &Bytes32,
    ) -> anyhow::Result<Option<Vec<FuelCoreReceipt>>> {
        Ok(self.receipts.clone())
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn doesnt_publish_any_message_when_no_block_has_been_mined() {
    let (blocks_broadcaster, __) = broadcast::channel::<ImporterResult>(1);
    let publisher = new_publisher(blocks_broadcaster.clone()).await;

    let shutdown_controller = start_publisher(&publisher).await;
    stop_publisher(shutdown_controller).await;

    assert!(publisher.get_streams().is_empty().await);
}

#[tokio::test(flavor = "multi_thread")]
async fn publishes_a_block_message_when_a_single_block_has_been_mined() {
    let (blocks_broadcaster, _) = broadcast::channel::<ImporterResult>(1);
    let publisher = new_publisher(blocks_broadcaster.clone()).await;

    publish_block(&publisher, &blocks_broadcaster).await;

    assert!(publisher
        .get_streams()
        .blocks
        .get_last_published(BlocksSubject::WILDCARD)
        .await
        .is_ok_and(|result| result.is_some()));
}

#[tokio::test(flavor = "multi_thread")]
async fn publishes_transaction_for_each_published_block() {
    let (blocks_broadcaster, _) = broadcast::channel::<ImporterResult>(1);
    let publisher = new_publisher(blocks_broadcaster.clone()).await;

    publish_block(&publisher, &blocks_broadcaster).await;

    assert!(publisher
        .get_streams()
        .transactions
        .get_last_published(TransactionsSubject::WILDCARD)
        .await
        .is_ok_and(|result| result.is_some()));
}

#[tokio::test(flavor = "multi_thread")]
async fn publishes_receipts() {
    let (blocks_broadcaster, _) = broadcast::channel::<ImporterResult>(1);

    let receipts = [
        FuelCoreReceipt::Call {
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
        FuelCoreReceipt::Return {
            id: ContractId::default(),
            val: 0,
            pc: 0,
            is: 0,
        },
        FuelCoreReceipt::ReturnData {
            id: ContractId::default(),
            ptr: 0,
            len: 0,
            digest: Bytes32::default(),
            pc: 0,
            is: 0,
            data: None,
        },
        FuelCoreReceipt::Revert {
            id: ContractId::default(),
            ra: 0,
            pc: 0,
            is: 0,
        },
        FuelCoreReceipt::Log {
            id: ContractId::default(),
            ra: 0,
            rb: 0,
            rc: 0,
            rd: 0,
            pc: 0,
            is: 0,
        },
        FuelCoreReceipt::LogData {
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
        FuelCoreReceipt::Transfer {
            id: ContractId::default(),
            to: ContractId::default(),
            amount: 0,
            asset_id: AssetId::default(),
            pc: 0,
            is: 0,
        },
        FuelCoreReceipt::TransferOut {
            id: ContractId::default(),
            to: Address::default(),
            amount: 0,
            asset_id: AssetId::default(),
            pc: 0,
            is: 0,
        },
        FuelCoreReceipt::Mint {
            sub_id: Bytes32::default(),
            contract_id: ContractId::default(),
            val: 0,
            pc: 0,
            is: 0,
        },
        FuelCoreReceipt::Burn {
            sub_id: Bytes32::default(),
            contract_id: ContractId::default(),
            val: 0,
            pc: 0,
            is: 0,
        },
    ];

    let fuel_core = TestFuelCore::default(blocks_broadcaster.clone())
        .with_receipts(receipts.to_vec())
        .arc();

    let publisher = Publisher::default(&nats_client().await, fuel_core)
        .await
        .unwrap();

    publish_block(&publisher, &blocks_broadcaster).await;

    let mut receipts_stream =
        publisher.get_streams().receipts.catchup(10).await.unwrap();

    let receipts: HashSet<Receipt> = receipts.iter().map(Into::into).collect();
    while let Some(Some(receipt)) = receipts_stream.next().await {
        assert!(receipts.contains(&receipt));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn publishes_inputs() {
    let (blocks_broadcaster, _) = broadcast::channel::<ImporterResult>(1);
    let publisher = new_publisher(blocks_broadcaster.clone()).await;

    publish_block(&publisher, &blocks_broadcaster).await;

    assert!(publisher
        .get_streams()
        .inputs
        .get_last_published(InputsByIdSubject::WILDCARD)
        .await
        .is_ok_and(|result| result.is_some()));
}

async fn new_publisher(broadcaster: Sender<ImporterResult>) -> Publisher {
    let fuel_core = TestFuelCore::default(broadcaster).arc();
    Publisher::default(&nats_client().await, fuel_core)
        .await
        .unwrap()
}

async fn publish_block(
    publisher: &Publisher,
    blocks_broadcaster: &Sender<ImporterResult>,
) {
    let shutdown_controller = start_publisher(&publisher).await;
    send_block(&blocks_broadcaster);
    stop_publisher(shutdown_controller).await;
}

async fn start_publisher(publisher: &Publisher) -> Arc<ShutdownController> {
    let shutdown_controller = ShutdownController::new().arc();
    let shutdown_token = shutdown_controller.get_token();
    tokio::spawn({
        let publisher = publisher.clone();
        async move {
            publisher.run(shutdown_token).await.unwrap();
        }
    });
    wait_for_publisher_to_start().await;
    shutdown_controller
}
async fn stop_publisher(shutdown_controller: Arc<ShutdownController>) {
    wait_for_publisher_to_process_block().await;

    assert!(shutdown_controller.initiate_shutdown().is_ok());
}

async fn wait_for_publisher_to_start() {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}
async fn wait_for_publisher_to_process_block() {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}

fn send_block(broadcaster: &Sender<ImporterResult>) {
    let block = create_test_block();
    assert!(broadcaster.send(block).is_ok());
}
fn create_test_block() -> ImporterResult {
    let mut block_entity = FuelCoreBlock::default();
    let tx = FuelCoreTransaction::default_test_tx();

    *block_entity.transactions_mut() = vec![tx];

    ImporterResult {
        shared_result: Arc::new(ImportResult {
            sealed_block: SealedBlock {
                entity: block_entity,
                ..Default::default()
            },
            ..Default::default()
        }),
        changes: Arc::new(HashMap::new()),
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
