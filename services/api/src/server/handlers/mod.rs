pub mod accounts;
pub mod blocks;
pub mod contracts;
pub mod inputs;
pub mod outputs;
pub mod receipts;
pub mod transactions;
pub mod utxos;
use actix_web::{http::StatusCode, web};
use fuel_streams_domains::{
    inputs::InputType,
    outputs::OutputType,
    receipts::ReceiptType,
};
use fuel_streams_store::db::DbItem;
use fuel_web_utils::{
    api_key::middleware::ApiKeyAuth,
    server::api::with_prefixed_route,
};
use serde::Serialize;

use super::handlers;
use crate::server::state::ServerState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Validation error {0}")]
    Validation(#[from] validator::ValidationErrors),
}

impl From<Error> for actix_web::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Sqlx(e) => actix_web::error::InternalError::new(
                e,
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into(),
            Error::Validation(e) => {
                actix_web::error::InternalError::new(e, StatusCode::BAD_REQUEST)
                    .into()
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDbEntityResponse<T: DbItem> {
    data: Vec<T>,
}

pub fn create_services(
    state: ServerState,
) -> impl Fn(&mut web::ServiceConfig) + Send + Sync + 'static {
    move |cfg: &mut web::ServiceConfig| {
        let api_key_middleware =
            ApiKeyAuth::new(&state.api_keys_manager, &state.db);
        cfg.app_data(web::Data::new(state.clone()));
        // blocks
        cfg.service(
        web::scope(&with_prefixed_route("blocks"))
                .wrap(api_key_middleware.clone())
                .route("", web::get().to({
                    move |req, query, state: web::Data<ServerState>| {
                        handlers::blocks::get_blocks(req, query, state)
                    }
                }))
                .route("/{height}/receipts", web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        handlers::blocks::get_block_receipts(req, path, query, state)
                    }
                }))
                .route("/{height}/transactions", web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        handlers::blocks::get_block_transactions(req, path, query, state)
                    }
                }))
                .route("/{height}/inputs", web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        handlers::blocks::get_block_inputs(req, path, query, state)
                    }
                }))
                .route("/{height}/outputs", web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        handlers::blocks::get_block_outputs(req, path, query, state)
                    }
                }))
        );

        // transactions
        cfg.service(
            web::scope(&with_prefixed_route("transactions"))
                    .wrap(api_key_middleware.clone())
                    .route("", web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::transactions::get_transactions(req, query, state)
                        }
                    }))
                    .route("/{tx_id}/receipts", web::get().to({
                        move |req, path, query, state: web::Data<ServerState>| {
                            handlers::transactions::get_transaction_receipts(req, path, query, state)
                        }
                    }))
                    .route("/{tx_id}/inputs", web::get().to({
                        move |req, path, query, state: web::Data<ServerState>| {
                            handlers::transactions::get_transaction_inputs(req, path, query, state)
                        }
                    }))
                    .route("/{tx_id}/outputs", web::get().to({
                        move |req, path, query, state: web::Data<ServerState>| {
                            handlers::transactions::get_transaction_outputs(req, path, query, state)
                        }
                    }))
        );

        // inputs
        cfg.service(
            web::scope(&with_prefixed_route("inputs"))
                .wrap(api_key_middleware.clone())
                .route(
                    "",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::inputs::get_inputs(
                                req, query, state, None,
                            )
                        }
                    }),
                )
                .route(
                    "/message",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::inputs::get_inputs(
                                req,
                                query,
                                state,
                                Some(InputType::Message),
                            )
                        }
                    }),
                )
                .route(
                    "/contract",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::inputs::get_inputs(
                                req,
                                query,
                                state,
                                Some(InputType::Contract),
                            )
                        }
                    }),
                )
                .route(
                    "/coin",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::inputs::get_inputs(
                                req,
                                query,
                                state,
                                Some(InputType::Coin),
                            )
                        }
                    }),
                ),
        );

        // outputs
        cfg.service(
            web::scope(&with_prefixed_route("outputs"))
                .wrap(api_key_middleware.clone())
                .route(
                    "",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::outputs::get_outputs(
                                req, query, state, None,
                            )
                        }
                    }),
                )
                .route(
                    "/change",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::outputs::get_outputs(
                                req,
                                query,
                                state,
                                Some(OutputType::Change),
                            )
                        }
                    }),
                )
                .route(
                    "/coin",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::outputs::get_outputs(
                                req,
                                query,
                                state,
                                Some(OutputType::Coin),
                            )
                        }
                    }),
                )
                .route(
                    "/contract",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::outputs::get_outputs(
                                req,
                                query,
                                state,
                                Some(OutputType::Contract),
                            )
                        }
                    }),
                )
                .route(
                    "/contract-created",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::outputs::get_outputs(
                                req,
                                query,
                                state,
                                Some(OutputType::ContractCreated),
                            )
                        }
                    }),
                )
                .route(
                    "/variable",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::outputs::get_outputs(
                                req,
                                query,
                                state,
                                Some(OutputType::Variable),
                            )
                        }
                    }),
                ),
        );

        // utxos
        cfg.service(
            web::scope(&with_prefixed_route("utxos"))
                .wrap(api_key_middleware.clone())
                .route(
                    "",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::utxos::get_utxos(req, query, state)
                        }
                    }),
                ),
        );

        // receipts
        cfg.service(
            web::scope(&with_prefixed_route("receipts"))
                .wrap(api_key_middleware.clone())
                .route(
                    "",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req, query, state, None,
                            )
                        }
                    }),
                )
                .route(
                    "/burn",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::Burn),
                            )
                        }
                    }),
                )
                .route(
                    "/mint",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::Mint),
                            )
                        }
                    }),
                )
                .route(
                    "/message-out",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::MessageOut),
                            )
                        }
                    }),
                )
                .route(
                    "/script-result",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::ScriptResult),
                            )
                        }
                    }),
                )
                .route(
                    "/transfer-out",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::TransferOut),
                            )
                        }
                    }),
                )
                .route(
                    "/transfer",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::Transfer),
                            )
                        }
                    }),
                )
                .route(
                    "/logdata",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::LogData),
                            )
                        }
                    }),
                )
                .route(
                    "/log",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::Log),
                            )
                        }
                    }),
                )
                .route(
                    "/revert",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::Revert),
                            )
                        }
                    }),
                )
                .route(
                    "/panic",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::Panic),
                            )
                        }
                    }),
                )
                .route(
                    "/return-data",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::ReturnData),
                            )
                        }
                    }),
                )
                .route(
                    "/return",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::Return),
                            )
                        }
                    }),
                )
                .route(
                    "/call",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            handlers::receipts::get_receipts(
                                req,
                                query,
                                state,
                                Some(ReceiptType::Call),
                            )
                        }
                    }),
                ),
        );

        // contracts
        cfg.service(
        web::scope(&with_prefixed_route("contracts"))
                .wrap(api_key_middleware.clone())
                .route("/{contract_id}/transactions", web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        handlers::contracts::get_contracts_transactions(req, path, query, state)
                    }
                }))
                .route("/{contract_id}/inputs", web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        handlers::contracts::get_contracts_inputs(req, path, query, state)
                    }
                }))
                .route("/{contract_id}/outputs", web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        handlers::contracts::get_contracts_outputs(req, path, query, state)
                    }
                }))
                .route("/{contract_id}/utxos", web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        handlers::contracts::get_contracts_utxos(req, path, query, state)
                    }
                }))
            );

        // accounts
        cfg.service(
            web::scope(&with_prefixed_route("accounts"))
                    .wrap(api_key_middleware.clone())
                    .route("/{address}/transactions", web::get().to({
                        move |req, path, query, state: web::Data<ServerState>| {
                            handlers::accounts::get_accounts_transactions(req, path, query, state)
                        }
                    }))
                    .route("/{address}/inputs", web::get().to({
                        move |req, path, query, state: web::Data<ServerState>| {
                            handlers::accounts::get_accounts_inputs(req, path, query, state)
                        }
                    }))
                    .route("/{address}/outputs", web::get().to({
                        move |req, path, query, state: web::Data<ServerState>| {
                            handlers::accounts::get_accounts_outputs(req, path, query, state)
                        }
                    }))
                    .route("/{address}/utxos", web::get().to({
                        move |req, path, query, state: web::Data<ServerState>| {
                            handlers::accounts::get_accounts_utxos(req, path, query, state)
                        }
                    }))
                );
    }
}
