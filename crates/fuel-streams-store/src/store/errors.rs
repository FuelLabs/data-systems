use crate::db::DbError;

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error(transparent)]
    Db(#[from] DbError),
}
