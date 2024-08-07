// File: crates/icn_market/src/market_maker.rs

use crate::entities::Resource;
use crate::order_book::Order;
use crate::matching_engine::MatchingEngine;
use std::collections::HashMap;

pub struct MarketMaker {
    id: String,
    inventory: HashMap<Resource, f64>,
    spread: f64,
}

impl MarketMaker {
    pub fn new(id: String, spread: f64) -> Self {
        MarketMaker {
            id,
            inventory: HashMap::new(),
            spread,
        }
    }

    pub fn update_inventory(&mut self, resource: Resource, quantity: f64) {
        *self.inventory.entry(resource).or_insert(0.0) += quantity;
    }

    pub fn generate_orders(&self, resource: &Resource, current_price: f64, quantity: f64) -> Vec<Order> {
        let mut orders = Vec::new();

        let bid_price = current_price * (1.0 - self.spread / 2.0);
        let ask_price = current_price * (1.0 + self.spread / 2.0);

        orders.push(Order {
            id: format!("bid_{}", self.id),
            trader_id: self.id.clone(),
            resource: resource.clone(),
            quantity,
            price: bid_price,
            is_buy: true,
            timestamp: chrono::Utc::now().timestamp() as u64,
        });

        orders.push(Order {
            id: format!("ask_{}", self.id),
            trader_id: self.id.clone(),
            resource: resource.clone(),
            quantity,
            price: ask_price,
            is_buy: false,
            timestamp: chrono::Utc::now().timestamp() as u64,
        });

        orders
    }

    pub fn adjust_inventory(&mut self, engine: &mut MatchingEngine, target_inventory: f64) {
        for (resource, &current_inventory) in &self.inventory {
            let inventory_difference = target_inventory - current_inventory;
            if inventory_difference.abs() > 1e-6 {
                let order = Order {
                    id: format!("adjust_{}", self.id),
                    trader_id: self.id.clone(),
                    resource: resource.clone(),
                    quantity: inventory_difference.abs(),
                    price: 0.0, // Market order
                    is_buy: inventory_difference > 0.0,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                };
                engine.process_order(order);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_maker() {
        let mut market_maker = MarketMaker::new("MM1".to_string(), 0.01);
        let resource = Resource {
            name: "TestResource".to_string(),
            quantity: 1000.0,
            unit: "units".to_string(),
        };

        market_maker.update_inventory(resource.clone(), 100.0);
        assert_eq!(*market_maker.inventory.get(&resource).unwrap(), 100.0);

        let orders = market_maker.generate_orders(&resource, 10.0, 5.0);
        assert_eq!(orders.len(), 2);
        assert!(orders[0].is_buy);
        assert!(!orders[1].is_buy);
        assert!(orders[0].price < 10.0);
        assert!(orders[1].price > 10.0);

        let mut engine = MatchingEngine::new();
        market_maker.adjust_inventory(&mut engine, 150.0);
        // Note: We can't directly test the result of adjust_inventory here
        // as it depends on the MatchingEngine's state. In a real scenario,
        // we would need to set up the engine with some opposing orders first.
    }
}