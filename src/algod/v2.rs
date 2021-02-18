use crate::error::AlgorandError;
use crate::models::{
    Account, Block, NodeStatus, PendingTransactions, Supply, Transaction,
    TransactionID, TransactionList, TransactionParams, Version,
};
use crate::transaction::SignedTransaction;
use crate::Round;
use reqwest::header::HeaderMap;

const AUTH_HEADER: &str = "X-Algo-API-Token";

/// Client for interacting with the Algorand protocol daemon
pub struct Client {
    pub(super) url: String,
    pub(super) token: String,
    pub(super) headers: HeaderMap,
    pub(super) http_client: reqwest::Client,
}

impl Client {
    /// Returns Ok if healthy
    pub fn health(&self) -> Result<(), AlgorandError> {
        let _ = self
            .http_client
            .get(&format!("{}health", self.url))
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?;
        Ok(())
    }

    /// Retrieves the current version
    pub fn versions(&self) -> Result<Version, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}versions", self.url))
            .headers(self.headers.clone())
            .header(AUTH_HEADER, &self.token)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets the current node status
    pub fn status(&self) -> Result<NodeStatus, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/status", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Waits for a block to appear after the specified round and returns the node status at the time
    pub fn status_after_block(&self, round: Round) -> Result<NodeStatus, AlgorandError> {
        let response = self
            .http_client
            .get(&format!(
                "{}v2/status/wait-for-block-after/{}",
                self.url, round.0
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Get the block for the given round
    pub fn block(&self, round: Round) -> Result<Block, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/blocks/{}", self.url, round.0))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets the current supply reported by the ledger
    pub fn ledger_supply(&self) -> Result<Supply, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/ledger/supply", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    pub fn account_information(&self, address: &str) -> Result<Account, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/accounts/{}", self.url, address))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets a list of unconfirmed transactions currently in the transaction pool
    ///
    /// Sorted by priority in decreasing order and truncated at the specified limit, or returns all if specified limit is 0
    pub fn pending_transactions(&self, limit: u64) -> Result<PendingTransactions, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/pending", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .query(&[("max", limit.to_string())])
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Get a specified pending transaction
    pub fn pending_transaction_information(
        &self,
        transaction_id: &str,
    ) -> Result<Transaction, AlgorandError> {
        let response = self
            .http_client
            .get(&format!(
                "{}v2/transactions/pending/{}",
                self.url, transaction_id
            ))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Get a list of confirmed transactions, limited to filters if specified
    pub fn transactions(
        &self,
        address: &str,
        first_round: Option<Round>,
        last_round: Option<Round>,
        from_date: Option<String>,
        to_date: Option<String>,
        limit: Option<u64>,
    ) -> Result<TransactionList, AlgorandError> {
        let mut query = Vec::new();
        if let Some(first_round) = first_round {
            query.push(("first-round", first_round.0.to_string()))
        }
        if let Some(last_round) = last_round {
            query.push(("last-round", last_round.0.to_string()))
        }
        if let Some(from_date) = from_date {
            query.push(("fromDate", from_date))
        }
        if let Some(to_date) = to_date {
            query.push(("toDate", to_date))
        }
        if let Some(limit) = limit {
            query.push(("max", limit.to_string()))
        }
        let response = self
            .http_client
            .get(&format!("{}v2/accounts/{}/transactions/pending/", self.url, address))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .query(&query)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Broadcasts a transaction to the network
    pub fn send_transaction(
        &self,
        signed_transaction: &SignedTransaction,
    ) -> Result<TransactionID, AlgorandError> {
        let bytes = rmp_serde::to_vec_named(signed_transaction)?;
        self.raw_transaction(&bytes)
    }

    /// Broadcasts a raw transaction to the network
    pub fn raw_transaction(&self, raw: &[u8]) -> Result<TransactionID, AlgorandError> {
        let response = self
            .http_client
            .post(&format!("{}v2/transactions", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .body(raw.to_vec())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }

    /// Gets parameters for constructing a new transaction
    pub fn transaction_params(&self) -> Result<TransactionParams, AlgorandError> {
        let response = self
            .http_client
            .get(&format!("{}v2/transactions/params", self.url))
            .header(AUTH_HEADER, &self.token)
            .headers(self.headers.clone())
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response)
    }
}
