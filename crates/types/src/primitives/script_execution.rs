use fuel_core_types::{fuel_asm::RawInstruction, fuel_tx::PanicReason};

use crate::fuel_core::*;

#[derive(
    Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub struct PanicInstruction {
    pub reason: PanicReason,
    pub instruction: RawInstruction,
}

impl utoipa::ToSchema for PanicInstruction {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("PanicInstruction")
    }
}

impl utoipa::PartialSchema for PanicInstruction {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::ObjectBuilder::new()
            .title(Some("PanicInstruction"))
            .description(Some("Instruction that caused a panic in the VM"))
            .property(
                "reason",
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String)
                    .enum_values(Some([
                        "UnknownPanicReason",
                        "Revert",
                        "OutOfGas",
                        "TransactionValidity",
                        "MemoryOverflow",
                        "ArithmeticOverflow",
                        "ContractNotFound",
                        "MemoryOwnership",
                        "NotEnoughBalance",
                        "ExpectedInternalContext",
                        "AssetIdNotFound",
                        "InputNotFound",
                        "OutputNotFound",
                        "WitnessNotFound",
                        "TransactionMaturity",
                        "InvalidMetadataIdentifier",
                        "MalformedCallStructure",
                        "ReservedRegisterNotWritable",
                        "InvalidFlags",
                        "InvalidImmediateValue",
                        "ExpectedCoinInput",
                        "EcalError",
                        "MemoryWriteOverlap",
                        "ContractNotInInputs",
                        "InternalBalanceOverflow",
                        "ContractMaxSize",
                        "ExpectedUnallocatedStack",
                        "MaxStaticContractsReached",
                        "TransferAmountCannotBeZero",
                        "ExpectedOutputVariable",
                        "ExpectedParentInternalContext",
                        "PredicateReturnedNonOne",
                        "ContractIdAlreadyDeployed",
                        "ContractMismatch",
                        "MessageDataTooLong",
                        "ArithmeticError",
                        "ContractInstructionNotAllowed",
                        "TransferZeroCoins",
                        "InvalidInstruction",
                        "MemoryNotExecutable",
                        "PolicyIsNotSet",
                        "PolicyNotFound",
                        "TooManyReceipts",
                        "BalanceOverflow",
                        "InvalidBlockHeight",
                        "TooManySlots",
                        "ExpectedNestedCaller",
                        "MemoryGrowthOverlap",
                        "UninitalizedMemoryAccess",
                        "OverridingConsensusParameters",
                        "UnknownStateTransactionBytecodeRoot",
                        "OverridingStateTransactionBytecode",
                        "BytecodeAlreadyUploaded",
                        "ThePartIsNotSequentiallyConnected",
                        "BlobNotFound",
                        "BlobIdAlreadyUploaded",
                        "GasCostNotDefined",
                        "UnsupportedCurveId",
                        "UnsupportedOperationType",
                        "InvalidEllipticCurvePoint",
                        "InputContractDoesNotExist",
                    ]))
                    .description(Some("Reason for VM panic"))
                    .build(),
            )
            .property(
                "instruction",
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(
                        utoipa::openapi::schema::SchemaFormat::KnownFormat(
                            utoipa::openapi::KnownFormat::Int32,
                        ),
                    ))
                    .description(Some(
                        "Raw instruction that caused the panic (u32)",
                    ))
                    .build(),
            )
            .required("reason")
            .required("instruction")
            .build()
            .into()
    }
}

impl From<FuelCorePanicInstruction> for PanicInstruction {
    fn from(value: FuelCorePanicInstruction) -> Self {
        Self {
            reason: value.reason().to_owned(),
            instruction: value.instruction().to_owned(),
        }
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
#[repr(u64)]
pub enum ScriptExecutionResult {
    Success,
    Revert,
    Panic,
    // Generic failure case since any u64 is valid here
    GenericFailure(u64),
    #[default]
    Unknown,
}
impl From<FuelCoreScriptExecutionResult> for ScriptExecutionResult {
    fn from(value: FuelCoreScriptExecutionResult) -> Self {
        match value {
            FuelCoreScriptExecutionResult::Success => Self::Success,
            FuelCoreScriptExecutionResult::Revert => Self::Revert,
            FuelCoreScriptExecutionResult::Panic => Self::Panic,
            FuelCoreScriptExecutionResult::GenericFailure(value) => {
                Self::GenericFailure(value)
            }
        }
    }
}
