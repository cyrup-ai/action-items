//! Transaction support with automatic commit/rollback and panic safety

use std::collections::HashMap;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use futures::FutureExt;
use serde::Serialize;
use surrealdb::engine::local::Db;
use surrealdb::{RecordId, Response, Surreal, Value};
use tracing::{debug, warn};

use crate::config::{DatabaseError, validate_table_name};
use crate::service::DatabaseService;

impl DatabaseService {
    /// Execute operations within a transaction with automatic commit/rollback
    ///
    /// The transaction is automatically committed if the closure returns Ok,
    /// or rolled back if the closure returns Err or panics.
    pub async fn with_transaction<T, F, Fut>(&self, f: F) -> Result<T, DatabaseError>
    where
        F: for<'a> FnOnce(&'a TransactionContext<'a>) -> Fut + Send,
        Fut: Future<Output = Result<T, DatabaseError>> + Send,
    {
        let db = self.db();
        
        // Begin transaction using real SurrealDB SDK API
        let transaction = db.transaction()
            .await
            .map_err(|e| DatabaseError::TransactionFailed(e.to_string()))?;

        // Create context for transaction operations
        let ctx = TransactionContext {
            transaction: &transaction,
        };

        // Execute user operations with panic handling
        let result = AssertUnwindSafe(f(&ctx)).catch_unwind().await;

        // Handle commit or rollback based on result
        match result {
            Ok(Ok(value)) => {
                // Commit transaction
                transaction.commit()
                    .await
                    .map_err(|e| DatabaseError::TransactionFailed(e.to_string()))?;

                Ok(value)
            },
            Ok(Err(error)) => {
                // Rollback transaction due to application error
                if let Err(cancel_err) = transaction.cancel().await {
                    warn!("Transaction rollback failed: {:?}", cancel_err);
                } else {
                    debug!("Transaction rolled back due to error");
                }

                Err(error)
            },
            Err(_panic) => {
                // Rollback transaction due to panic
                if let Err(cancel_err) = transaction.cancel().await {
                    warn!("Transaction rollback failed after panic: {:?}", cancel_err);
                } else {
                    debug!("Transaction rolled back due to panic");
                }

                Err(DatabaseError::TransactionFailed(
                    "Transaction closure panicked".to_string(),
                ))
            },
        }
    }
}

/// Context provided to transaction closures for database operations
pub struct TransactionContext<'a> {
    transaction: &'a Surreal<Db>,
}

impl<'a> TransactionContext<'a> {
    /// Create a record in a table within the transaction
    pub async fn create<T>(&self, table: &str, data: T) -> Result<RecordId, DatabaseError>
    where
        T: Serialize + Send + 'static,
    {
        validate_table_name(table)?;

        let result: Result<Option<RecordId>, _> = self.transaction.create(table).content(data).await;

        match result {
            Ok(Some(record)) => Ok(record),
            Ok(None) => Err(DatabaseError::QueryFailed("No record created".to_string())),
            Err(e) => Err(DatabaseError::QueryFailed(e.to_string())),
        }
    }

    /// Select records from a table within the transaction
    pub async fn select<T>(&self, table: &str) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        validate_table_name(table)?;

        let result: Result<Vec<T>, _> = self.transaction.select(table).await;

        result.map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    /// Update a record by ID within the transaction
    pub async fn update<T>(&self, thing: &RecordId, data: T) -> Result<Option<T>, DatabaseError>
    where
        T: Serialize + for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        let result: Result<Vec<T>, _> = self.transaction.update(thing.to_string()).content(data).await;

        match result {
            Ok(mut records) if !records.is_empty() => Ok(Some(records.remove(0))),
            Ok(_) => Ok(None),
            Err(e) => Err(DatabaseError::QueryFailed(e.to_string())),
        }
    }

    /// Delete a record by ID within the transaction
    pub async fn delete(&self, thing: &RecordId) -> Result<Option<Value>, DatabaseError> {
        let result: Result<Vec<Value>, _> = self.transaction.delete(thing.to_string()).await;

        match result {
            Ok(mut records) if !records.is_empty() => Ok(Some(records.remove(0))),
            Ok(_) => Ok(None),
            Err(e) => Err(DatabaseError::QueryFailed(e.to_string())),
        }
    }

    /// Execute a raw SurrealQL query within the transaction
    pub async fn query(&self, sql: &str) -> Result<Response, DatabaseError> {
        self.transaction.query(sql).await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    /// Execute a query with parameters within the transaction
    pub async fn query_with_params(
        &self,
        sql: &str,
        params: HashMap<String, Value>,
    ) -> Result<Response, DatabaseError> {
        self.transaction.query(sql).bind(params).await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }
}