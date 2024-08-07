// File: crates/icn_market/src/order_book.rs

use std::collections::{BTreeMap, VecDeque};
use crate::entities::Resource;
use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub trader_id: String,
    pub resource: Resource,
    pub quantity: f64,
    pub price: f64,
    pub is_buy: bool,
    pub timestamp: u64,
}

pub struct OrderBook {
    buy_orders: BTreeMap<f64, VecDeque<Order>>,
    sell_orders: BTreeMap<f64, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            buy_orders: BTreeMap::new(),
            sell_orders: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let orders = if order.is_buy {
            &mut self.buy_orders
        } else {
            &mut self.sell_orders
        };

        orders.entry(order.price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    pub fn remove_order(&mut self, id: &str, is_buy: bool) -> Option<Order> {
        let orders = if is_buy {
            &mut self.buy_orders
        } else {
            &mut self.sell_orders
        };

        for (_, queue) in orders.iter_mut() {
            if let Some(pos) = queue.iter().position(|order| order.id == id) {
                return Some(queue.remove(pos).unwrap());
            }
        }
        None
    }

    pub fn get_best_bid(&self) -> Option<&Order> {
        self.buy_orders.iter().next_back().and_then(|(_, queue)| queue.front())
    }

    pub fn get_best_ask(&self) -> Option<&Order> {
        self.sell_orders.iter().next().and_then(|(_, queue)| queue.front())
    }
}