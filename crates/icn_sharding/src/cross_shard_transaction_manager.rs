use crate::blockchain::Transaction;
use crate::sharding::ShardingManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

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
    pending_transactions: HashMap<String, CrossShardTransaction>,
    tx_channels: HashMap<u64, mpsc::Sender<CrossShardTransaction>>,
}

impl CrossShardCommunicator {
    pub fn new(sharding_manager: Arc<Mutex<ShardingManager>>) -> Self {
        let mut tx_channels = HashMap::new();
        let shard_count = sharding_manager.lock().unwrap().get_shard_count();
        for i in 0..shard_count {
            let (tx, mut rx) = mpsc::channel(100);
            tx_channels.insert(i, tx);
            let sm = Arc::clone(&sharding_manager);
            tokio::spawn(async move {
                while let Some(transaction) = rx.recv().await {
                    if let Err(e) = Self::process_transaction(sm.clone(), transaction).await {
                        eprintln!("Error processing cross-shard transaction: {}", e);
                    }
                }
            });
        }

        CrossShardCommunicator {
            sharding_manager,
            pending_transactions: HashMap::new(),
            tx_channels,
        }
    }

    pub async fn initiate_cross_shard_transaction(&mut self, transaction: Transaction) -> Result<String, String> {
        let sharding_manager = self.sharding_manager.lock().unwrap();
        let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);

        if from_shard == to_shard {
            return Err("Not a cross-shard transaction".to_string());
        }

        let cross_shard_tx = CrossShardTransaction {
            transaction: transaction.clone(),
            from_shard,
            to_shard,
            status: CrossShardTransactionStatus::Initiated,
        };

        let tx_id = Uuid::new_v4().to_string();
        self.pending_transactions.insert(tx_id.clone(), cross_shard_tx.clone());

        if let Some(tx) = self.tx_channels.get(&from_shard) {
            tx.send(cross_shard_tx).await.map_err(|e| e.to_string())?;
        } else {
            return Err(format!("Channel for shard {} not found", from_shard));
        }

        Ok(tx_id)
    }

    async fn process_transaction(sharding_manager: Arc<Mutex<ShardingManager>>, mut transaction: CrossShardTransaction) -> Result<(), String> {
        // Phase 1: Lock funds in the source shard
        {
            let mut sm = sharding_manager.lock().unwrap();
            sm.lock_funds(&transaction.transaction.from, &transaction.transaction.currency_type, transaction.transaction.amount, transaction.from_shard)?;
        }
        transaction.status = CrossShardTransactionStatus::LockAcquired;

        // Phase 2: Commit the transaction in the destination shard
        {
            let mut sm = sharding_manager.lock().unwrap();
            sm.add_balance(&transaction.transaction.to, transaction.transaction.currency_type.clone(), transaction.transaction.amount)?;
        }

        // Phase 3: Finalize by removing the lock in the source shard
        {
            let mut sm = sharding_manager.lock().unwrap();
            sm.remove_fund_lock(&transaction.transaction.from, &transaction.transaction.currency_type, transaction.transaction.amount, transaction.from_shard)?;
        }

        transaction.status = CrossShardTransactionStatus::Committed;
        Ok(())
    }

    pub fn get_transaction_status(&self, tx_id: &str) -> Option<CrossShardTransactionStatus> {
        self.pending_transactions.get(tx_id).map(|tx| tx.status.clone())
    }
}