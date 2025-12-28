use convex::{ConvexClient, FunctionResult, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{Error, Result};

/// Wrapper around ConvexClient for ByteHub operations
#[derive(Clone)]
pub struct ConvexDb {
    client: Arc<RwLock<ConvexClient>>,
}

impl ConvexDb {
    /// Create a new ConvexDb instance
    pub async fn new(url: &str) -> Result<Self> {
        let client = ConvexClient::new(url)
            .await
            .map_err(|e| Error::Database(format!("Failed to connect to Convex: {}", e)))?;

        Ok(Self {
            client: Arc::new(RwLock::new(client)),
        })
    }

    /// Execute a query and return JSON value
    pub async fn query(
        &self,
        name: &str,
        args: BTreeMap<String, Value>,
    ) -> Result<serde_json::Value> {
        let mut client = self.client.write().await;
        let result = client
            .query(name, args)
            .await
            .map_err(|e| Error::Database(format!("Query failed: {}", e)))?;

        match result {
            FunctionResult::Value(v) => Ok(v.export()),
            FunctionResult::ErrorMessage(msg) => Err(Error::Database(msg)),
            FunctionResult::ConvexError(e) => Err(Error::Database(e.message)),
        }
    }

    /// Execute a mutation and return JSON value
    pub async fn mutation(
        &self,
        name: &str,
        args: BTreeMap<String, Value>,
    ) -> Result<serde_json::Value> {
        let mut client = self.client.write().await;
        let result = client
            .mutation(name, args)
            .await
            .map_err(|e| Error::Database(format!("Mutation failed: {}", e)))?;

        match result {
            FunctionResult::Value(v) => Ok(v.export()),
            FunctionResult::ErrorMessage(msg) => Err(Error::Database(msg)),
            FunctionResult::ConvexError(e) => Err(Error::Database(e.message)),
        }
    }

    /// Execute an action and return JSON value
    #[allow(dead_code)]
    pub async fn action(
        &self,
        name: &str,
        args: BTreeMap<String, Value>,
    ) -> Result<serde_json::Value> {
        let mut client = self.client.write().await;
        let result = client
            .action(name, args)
            .await
            .map_err(|e| Error::Database(format!("Action failed: {}", e)))?;

        match result {
            FunctionResult::Value(v) => Ok(v.export()),
            FunctionResult::ErrorMessage(msg) => Err(Error::Database(msg)),
            FunctionResult::ConvexError(e) => Err(Error::Database(e.message)),
        }
    }
}
