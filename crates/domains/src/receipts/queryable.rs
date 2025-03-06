use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use sea_query::{
    Asterisk,
    Condition,
    Expr,
    Iden,
    Order,
    PostgresQueryBuilder,
    Query,
    SelectStatement,
};
use serde::{Deserialize, Serialize};

use super::{ReceiptDbItem, ReceiptType};
use crate::queryable::Queryable;

#[allow(dead_code)]
#[derive(Iden)]
enum Receipts {
    #[iden = "receipts"]
    Table,
    #[iden = "subject"]
    Subject,
    #[iden = "block_height"]
    BlockHeight,
    #[iden = "tx_id"]
    TxId,
    #[iden = "tx_index"]
    TxIndex,
    #[iden = "receipt_index"]
    ReceiptIndex,
    #[iden = "receipt_type"]
    ReceiptType,
    #[iden = "from_contract_id"] //  ContractId for call/transfer/transfer_out
    FromContractId,
    #[iden = "to_contract_id"] // ContractId for call/transfer
    ToContractId,
    #[iden = "to_address"] // Address for transfer_out
    ToAddress,
    #[iden = "asset_id"] // for call/transfer/transfer_out
    ReceiptAssetId,
    #[iden = "contract_id"]
    // ContractId for return/return_data/panic/revert/log/log_data/mint/burn
    ReceiptContractId,
    #[iden = "sub_id"] // for mint/burn
    ReceiptSubId,
    #[iden = "sender_address"] //  Address for message_out
    ReceiptSenderAddress,
    #[iden = "recipient_address"] // Address for message_out
    ReceiptRecipientAddress,
    #[iden = "value"]
    Value,
}

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptsQuery {
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<i32>,
    pub receipt_type: Option<ReceiptType>,
    pub block_height: Option<BlockHeight>,
    pub from: Option<ContractId>,
    pub to: Option<ContractId>,
    pub contract: Option<ContractId>,
    pub asset: Option<AssetId>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
    pub sub_id: Option<Bytes32>,
    pub after: Option<i32>,
    pub before: Option<i32>,
    pub first: Option<i32>,
    pub last: Option<i32>,
    pub address: Option<Address>, // for the accounts endpoint
}

impl ReceiptsQuery {
    pub fn set_address(&mut self, address: &str) {
        self.address = Some(Address::from(address));
    }

    pub fn set_receipt_type(&mut self, receipt_type: Option<ReceiptType>) {
        self.receipt_type = receipt_type;
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.block_height = Some(height.into());
    }

    pub fn set_tx_id(&mut self, tx_id: &str) {
        self.tx_id = Some(tx_id.into());
    }

    pub fn get_sql_and_values(&self) -> (String, sea_query::Values) {
        self.build_query().build(PostgresQueryBuilder)
    }

    fn build_condition(&self) -> Condition {
        let mut condition = Condition::all();

        // handle address query
        if let Some(address) = &self.address {
            match self.receipt_type {
                Some(ReceiptType::Call) => {
                    condition = condition.add(
                        Expr::col(Receipts::ToAddress)
                            .eq(address.to_string())
                            .or(Expr::col(Receipts::ReceiptAssetId)
                                .eq(address.to_string())),
                    );
                }
                Some(ReceiptType::Panic)
                | Some(ReceiptType::Mint)
                | Some(ReceiptType::Burn) => {
                    condition = condition.add(
                        Expr::col(Receipts::ReceiptContractId)
                            .eq(address.to_string()),
                    );
                }
                Some(ReceiptType::Transfer)
                | Some(ReceiptType::TransferOut) => {
                    condition = condition.add(
                        Expr::col(Receipts::ReceiptAssetId)
                            .eq(address.to_string())
                            .or(Expr::col(Receipts::ToAddress)
                                .eq(address.to_string())),
                    );
                }
                Some(ReceiptType::MessageOut) => {
                    condition = condition.add(
                        Expr::col(Receipts::ReceiptSenderAddress)
                            .eq(address.to_string())
                            .or(Expr::col(Receipts::ReceiptRecipientAddress)
                                .eq(address.to_string())),
                    );
                }
                _ => {
                    condition = condition.add(
                        Expr::col(Receipts::ToAddress)
                            .eq(address.to_string())
                            .or(Expr::col(Receipts::ReceiptAssetId)
                                .eq(address.to_string()))
                            .or(Expr::col(Receipts::ReceiptContractId)
                                .eq(address.to_string()))
                            .or(Expr::col(Receipts::ReceiptSenderAddress)
                                .eq(address.to_string()))
                            .or(Expr::col(Receipts::ReceiptRecipientAddress)
                                .eq(address.to_string())),
                    );
                }
            }
        }

        if let Some(block_height) = &self.block_height {
            condition = condition
                .add(Expr::col(Receipts::BlockHeight).eq(**block_height));
        }

        if let Some(tx_id) = &self.tx_id {
            condition =
                condition.add(Expr::col(Receipts::TxId).eq(tx_id.to_string()));
        }

        if let Some(tx_index) = &self.tx_index {
            condition =
                condition.add(Expr::col(Receipts::TxIndex).eq(*tx_index));
        }

        if let Some(receipt_index) = &self.receipt_index {
            condition = condition
                .add(Expr::col(Receipts::ReceiptIndex).eq(*receipt_index));
        }

        if let Some(receipt_type) = &self.receipt_type {
            condition = condition.add(
                Expr::col(Receipts::ReceiptType).eq(receipt_type.to_string()),
            );
        }

        if let Some(from) = &self.from {
            condition = condition
                .add(Expr::col(Receipts::FromContractId).eq(from.to_string()));
        }

        if let Some(to) = &self.to {
            condition = condition
                .add(Expr::col(Receipts::ToContractId).eq(to.to_string()));
        }

        if let Some(contract) = &self.contract {
            condition = condition.add(
                Expr::col(Receipts::ReceiptContractId).eq(contract.to_string()),
            );
        }

        if let Some(asset) = &self.asset {
            condition = condition
                .add(Expr::col(Receipts::ReceiptAssetId).eq(asset.to_string()));
        }

        if let Some(sub_id) = &self.sub_id {
            condition = condition
                .add(Expr::col(Receipts::ReceiptSubId).eq(sub_id.to_string()));
        }

        if let Some(sender) = &self.sender {
            condition = condition.add(
                Expr::col(Receipts::ReceiptSenderAddress)
                    .eq(sender.to_string()),
            );
        }

        if let Some(recipient) = &self.recipient {
            condition = condition.add(
                Expr::col(Receipts::ReceiptRecipientAddress)
                    .eq(recipient.to_string()),
            );
        }

        condition
    }

    pub fn build_query(&self) -> SelectStatement {
        let mut condition = self.build_condition();

        // Add after/before conditions
        if let Some(after) = self.after {
            condition =
                condition.add(Expr::col(Receipts::BlockHeight).gt(after));
        }

        if let Some(before) = self.before {
            condition =
                condition.add(Expr::col(Receipts::BlockHeight).lt(before));
        }

        let mut query_builder = Query::select();
        let mut query = query_builder
            .column(Asterisk)
            .from(Receipts::Table)
            .cond_where(condition);

        // Add first/last conditions
        if let Some(first) = self.first {
            query = query
                .order_by(Receipts::BlockHeight, Order::Asc)
                .limit(first as u64);
        } else if let Some(last) = self.last {
            query = query
                .order_by(Receipts::BlockHeight, Order::Desc)
                .limit(last as u64);
        }

        query.to_owned()
    }
}

#[async_trait::async_trait]
impl Queryable for ReceiptsQuery {
    type Record = ReceiptDbItem;

    fn query_to_string(&self) -> String {
        self.build_query().to_string(PostgresQueryBuilder)
    }

    async fn execute<'c, E>(
        &self,
        executor: E,
    ) -> Result<Vec<ReceiptDbItem>, sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let sql = self.build_query().to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, ReceiptDbItem>(&sql)
            .fetch_all(executor)
            .await
    }
}

#[cfg(test)]
mod test {
    use fuel_streams_types::{
        Address,
        AssetId,
        BlockHeight,
        Bytes32,
        ContractId,
        TxId,
    };
    use pretty_assertions::assert_eq;

    use crate::{
        queryable::Queryable,
        receipts::queryable::{ReceiptType, ReceiptsQuery},
    };

    // Test constants
    const AFTER_POINTER: i32 = 10000;
    const BEFORE_POINTER: i32 = 20000;
    const FIRST_POINTER: i32 = 300;
    const LAST_POINTER: i32 = 400;
    const TEST_BLOCK_HEIGHT: i32 = 55;
    const TEST_TX_INDEX: u32 = 3;
    const TEST_RECEIPT_INDEX: i32 = 7;
    const TEST_TX_ID: &str =
        "0x0101010101010101010101010101010101010101010101010101010101010101";
    const TEST_CONTRACT_ID: &str =
        "0x0202020202020202020202020202020202020202020202020202020202020202";
    const TEST_ASSET_ID: &str =
        "0x0303030303030303030303030303030303030303030303030303030303030303";
    const TEST_ADDRESS: &str =
        "0x0404040404040404040404040404040404040404040404040404040404040404";
    const TEST_SUB_ID: &str =
        "0x0505050505050505050505050505050505050505050505050505050505050505";

    #[test]
    fn test_sql_with_fixed_conds() {
        // Test 1: basic query with tx_id, block_height and receipt_type
        let query = ReceiptsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: Some(BlockHeight::from(TEST_BLOCK_HEIGHT)),
            receipt_type: Some(ReceiptType::Call),
            tx_index: None,
            receipt_index: None,
            from: None,
            to: None,
            contract: None,
            asset: None,
            sender: None,
            recipient: None,
            sub_id: None,
            after: None,
            before: None,
            first: None,
            last: None,
            address: None,
        };

        assert_eq!(
            query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"block_height\" = {} AND \"tx_id\" = '{}' AND \"receipt_type\" = 'call'",
                TEST_BLOCK_HEIGHT, TEST_TX_ID)
        );

        // Test 2: query with contract filters and first pagination
        let contract_query = ReceiptsQuery {
            tx_id: None,
            block_height: None,
            receipt_type: None,
            tx_index: None,
            receipt_index: None,
            from: Some(ContractId::from(TEST_CONTRACT_ID)),
            to: Some(ContractId::from(TEST_CONTRACT_ID)),
            contract: None,
            asset: None,
            sender: None,
            recipient: None,
            sub_id: None,
            after: None,
            before: None,
            first: Some(FIRST_POINTER),
            last: None,
            address: None,
        };

        assert_eq!(
            contract_query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"from_contract_id\" = '{}' AND \"to_contract_id\" = '{}' ORDER BY \"block_height\" ASC LIMIT {}",
                TEST_CONTRACT_ID, TEST_CONTRACT_ID, FIRST_POINTER)
        );

        // Test 3: query with asset and last pagination with range
        let asset_query = ReceiptsQuery {
            tx_id: None,
            block_height: None,
            receipt_type: None,
            tx_index: None,
            receipt_index: None,
            from: None,
            to: None,
            contract: None,
            asset: Some(AssetId::from(TEST_ASSET_ID)),
            sender: None,
            recipient: None,
            sub_id: None,
            after: Some(AFTER_POINTER),
            before: None,
            first: None,
            last: Some(LAST_POINTER),
            address: None,
        };

        assert_eq!(
            asset_query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"asset_id\" = '{}' AND \"block_height\" > {} ORDER BY \"block_height\" DESC LIMIT {}",
                TEST_ASSET_ID, AFTER_POINTER, LAST_POINTER)
        );

        // Test 4: query with address filters and before condition
        let address_query = ReceiptsQuery {
            tx_id: None,
            block_height: None,
            receipt_type: None,
            tx_index: None,
            receipt_index: None,
            from: None,
            to: None,
            contract: None,
            asset: None,
            sender: Some(Address::from([4u8; 32])),
            recipient: Some(Address::from([4u8; 32])),
            sub_id: None,
            after: None,
            before: Some(BEFORE_POINTER),
            first: Some(FIRST_POINTER),
            last: None,
            address: None,
        };

        assert_eq!(
            address_query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"sender_address\" = '{}' AND \"recipient_address\" = '{}' AND \"block_height\" < {} ORDER BY \"block_height\" ASC LIMIT {}",
                TEST_ADDRESS, TEST_ADDRESS, BEFORE_POINTER, FIRST_POINTER)
        );

        // Test 5: query with detailed transaction indices
        let tx_details_query = ReceiptsQuery {
            tx_id: Some(TxId::from(TEST_TX_ID)),
            block_height: None,
            receipt_type: None,
            tx_index: Some(TEST_TX_INDEX),
            receipt_index: Some(TEST_RECEIPT_INDEX),
            from: None,
            to: None,
            contract: None,
            asset: None,
            sender: None,
            recipient: None,
            sub_id: None,
            after: None,
            before: None,
            first: None,
            last: None,
            address: None,
        };

        assert_eq!(
            tx_details_query.query_to_string(),
            format!("SELECT * FROM \"receipts\" WHERE \"tx_id\" = '{}' AND \"tx_index\" = {} AND \"receipt_index\" = {}",
                TEST_TX_ID, TEST_TX_INDEX, TEST_RECEIPT_INDEX)
        );
    }

    #[test]
    fn test_receipts_query_from_query_string() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&txIndex={}&receiptIndex={}&receiptType=Call&blockHeight={}&from={}&to={}&contract={}&asset={}&sender={}&recipient={}&subId={}&after={}&before={}&first={}&last={}",
            TEST_TX_ID,
            TEST_TX_INDEX,
            TEST_RECEIPT_INDEX,
            TEST_BLOCK_HEIGHT,
            TEST_CONTRACT_ID,
            TEST_CONTRACT_ID,
            TEST_CONTRACT_ID,
            TEST_ASSET_ID,
            TEST_ADDRESS,
            TEST_ADDRESS,
            TEST_SUB_ID,
            AFTER_POINTER,
            BEFORE_POINTER,
            FIRST_POINTER,
            LAST_POINTER
        );

        let query: ReceiptsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, Some(TEST_TX_INDEX));
        assert_eq!(query.receipt_index, Some(TEST_RECEIPT_INDEX));
        assert_eq!(query.receipt_type, Some(ReceiptType::Call));
        assert_eq!(
            query.block_height,
            Some(BlockHeight::from(TEST_BLOCK_HEIGHT))
        );
        assert_eq!(query.from, Some(ContractId::from(TEST_CONTRACT_ID)));
        assert_eq!(query.to, Some(ContractId::from(TEST_CONTRACT_ID)));
        assert_eq!(query.contract, Some(ContractId::from(TEST_CONTRACT_ID)));
        assert_eq!(query.asset, Some(AssetId::from(TEST_ASSET_ID)));
        assert_eq!(query.sender, Some(Address::from(TEST_ADDRESS)));
        assert_eq!(query.recipient, Some(Address::from(TEST_ADDRESS)));
        assert_eq!(query.sub_id, Some(Bytes32::from(TEST_SUB_ID)));
        assert_eq!(query.after, Some(AFTER_POINTER));
        assert_eq!(query.before, Some(BEFORE_POINTER));
        assert_eq!(query.first, Some(FIRST_POINTER));
        assert_eq!(query.last, Some(LAST_POINTER));
    }

    #[test]
    fn test_receipts_query_from_query_string_partial() {
        use serde_urlencoded;

        let query_string = format!(
            "txId={}&receiptType=Burn&contract={}&asset={}&subId={}&after={}&first={}",
            TEST_TX_ID,
            TEST_CONTRACT_ID,
            TEST_ASSET_ID,
            TEST_SUB_ID,
            AFTER_POINTER,
            FIRST_POINTER
        );

        let query: ReceiptsQuery =
            serde_urlencoded::from_str(&query_string).unwrap();

        assert_eq!(query.tx_id, Some(TxId::from(TEST_TX_ID)));
        assert_eq!(query.tx_index, None);
        assert_eq!(query.receipt_index, None);
        assert_eq!(query.receipt_type, Some(ReceiptType::Burn));
        assert_eq!(query.block_height, None);
        assert_eq!(query.from, None);
        assert_eq!(query.to, None);
        assert_eq!(query.contract, Some(ContractId::from(TEST_CONTRACT_ID)));
        assert_eq!(query.asset, Some(AssetId::from(TEST_ASSET_ID)));
        assert_eq!(query.sender, None);
        assert_eq!(query.recipient, None);
        assert_eq!(query.sub_id, Some(Bytes32::from(TEST_SUB_ID)));
        assert_eq!(query.after, Some(AFTER_POINTER));
        assert_eq!(query.before, None);
        assert_eq!(query.first, Some(FIRST_POINTER));
        assert_eq!(query.last, None);
    }
}
