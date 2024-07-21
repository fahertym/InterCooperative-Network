use crate::blockchain::Transaction;
use crate::sharding::ShardingManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;
use icn_common::{Error, Result};

#[derive(Clone, Debug)]
pub struct CrossShardTransaction {
    pub transaction: Transaction,
    pub from_shard: u64,
    pub to_shard: u64,
    pub status: CrossShardTransactionStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CrossShardTransactionStatus {
    Initiated,
    LockAcquired,
    Committed,
    Failed(String),
}

pub struct CrossShardCommunicator {
    sharding_manager: Arc<Mutex<ShardingManager>>,
    pending_transactions: Arc<Mutex<HashMap<String, CrossShardTransaction>>>,
    tx_channels: HashMap<u64, mpsc::Sender<CrossShardTransaction>>,
}

impl CrossShardCommunicator {
    pub fn new(sharding_manager: Arc<Mutex<ShardingManager>>) -> Self {
        let pending_transactions = Arc::new(Mutex::new(HashMap::new()));
        let mut tx_channels = HashMap::new();
        let shard_count = sharding_manager.lock().unwrap().get_shard_count();
        
        for i in 0..shard_count {
            let (tx, mut rx) = mpsc::channel(100);
            tx_channels.insert(i, tx);
            let sm = Arc::clone(&sharding_manager);
            let pt = Arc::clone(&pending_transactions);
            
            tokio::spawn(async move {
                while let Some(transaction) = rx.recv().await {
                    if let Err(e) = Self::process_transaction(sm.clone(), pt.clone(), transaction).await {
                        log::error!("Error processing cross-shard transaction: {}", e);
                    }
                }
            });
        }

        CrossShardCommunicator {
            sharding_manager,
            pending_transactions,
            tx_channels,
        }
    }

    pub async fn initiate_cross_shard_transaction(&self, transaction: Transaction) -> Result<String> {
        let sharding_manager = self.sharding_manager.lock().await;
        let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);

        if from_shard == to_shard {
            return Err(Error::ShardingError("Not a cross-shard transaction".to_string()));
        }

        let cross_shard_tx = CrossShardTransaction {
            transaction: transaction.clone(),
            from_shard,
            to_shard,
            status: CrossShardTransactionStatus::Initiated,
        };

        let tx_id = Uuid::new_v4().to_string();
        self.pending_transactions.lock().await.insert(tx_id.clone(), cross_shard_tx.clone());

        if let Some(tx) = self.tx_channels.get(&from_shard) {
            tx.send(cross_shard_tx).await.map_err(|e| Error::CommunicationError(e.to_string()))?;
        } else {
            return Err(Error::ShardingError(format!("Channel for shard {} not found", from_shard)));
        }

        Ok(tx_id)
    }

    async fn process_transaction(
        sharding_manager: Arc<Mutex<ShardingManager>>,
        pending_transactions: Arc<Mutex<HashMap<String, CrossShardTransaction>>>,
        mut transaction: CrossShardTransaction
    ) -> Result<()> {
        // Phase 1: Lock funds in the source shard
        {
            let mut sm = sharding_manager.lock().await;
            sm.lock_funds_in_shard(
                transaction.from_shard,
                &transaction.transaction.from,
                &transaction.transaction.currency_type,
                transaction.transaction.amount
            )?;
        }
        transaction.status = CrossShardTransactionStatus::LockAcquired;

        // Phase 2: Transfer funds to the destination shard
        {
            let mut sm = sharding_manager.lock().await;
            sm.transfer_between_shards(transaction.from_shard, transaction.to_shard, &transaction.transaction)?;
        }

        // Phase 3: Commit the transaction
        {
            let mut sm = sharding_manager.lock().await;
            sm.commit_cross_shard_transaction(&transaction.transaction, transaction.from_shard, transaction.to_shard)?;
        }

        transaction.status = CrossShardTransactionStatus::Committed;
        pending_transactions.lock().await.insert(Uuid::new_v4().to_string(), transaction);
        Ok(())
    }

    pub async fn get_transaction_status(&self, tx_id: &str) -> Option<CrossShardTransactionStatus> {
        self.pending_transactions.lock().await.get(tx_id).map(|tx| tx.status.clone())
    }

    pub async fn wait_for_transaction(&self, tx_id: &str, timeout: std::time::Duration) -> Result<CrossShardTransactionStatus> {
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < timeout {
            if let Some(status) = self.get_transaction_status(tx_id).await {
                match status {
                    CrossShardTransactionStatus::Committed => return Ok(status),
                    CrossShardTransactionStatus::Failed(reason) => return Err(Error::TransactionFailed(reason)),
                    _ => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        Err(Error::Timeout("Transaction timeout".to_string()))
    }

    pub async fn cleanup_completed_transactions(&self) {
        let mut pending_transactions = self.pending_transactions.lock().await;
        pending_transactions.retain(|_, tx| {
            match tx.status {
                CrossShardTransactionStatus::Committed => false,
                CrossShardTransactionStatus::Failed(_) => false,
                _ => true,
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::CurrencyType;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[tokio::test]
    async fn test_cross_shard_transaction() {
        let sharding_manager = Arc::new(Mutex::new(ShardingManager::new(2, 10)));
        let communicator = CrossShardCommunicator::new(sharding_manager.clone());

        {
            let mut sm = sharding_manager.lock().await;
            sm.add_address_to_shard("Alice".to_string(), 0);
            sm.add_address_to_shard("Bob".to_string(), 1);
            sm.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0).unwrap();
        }

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            200.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        let tx_id = communicator.initiate_cross_shard_transaction(transaction).await.unwrap();

        // Wait for the transaction to be processed
        let status = communicator.wait_for_transaction(&tx_id, std::time::Duration::from_secs(5)).await.unwrap();
        assert_eq!(status, CrossShardTransactionStatus::Committed);

        let sm = sharding_manager.lock().await;
        let alice_balance = sm.get_balance("Alice".to_string(), CurrencyType::BasicNeeds).unwrap();
        let bob_balance = sm.get_balance("Bob".to_string(), CurrencyType::BasicNeeds).unwrap();
        
        assert_eq!(alice_balance, 800.0);
        assert_eq!(bob_balance, 200.0);
    }
}