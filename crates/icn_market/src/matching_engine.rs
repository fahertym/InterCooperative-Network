// File: crates/icn_market/src/matching_engine.rs

use crate::order_book::{OrderBook, Order};
use crate::transaction::Transaction;

pub struct MatchingEngine {
    order_book: OrderBook,
}

impl MatchingEngine {
    pub fn new() -> Self {
        MatchingEngine {
            order_book: OrderBook::new(),
        }
    }

    pub fn process_order(&mut self, order: Order) -> Vec<Transaction> {
        let mut transactions = Vec::new();

        if order.is_buy {
            self.match_buy_order(order, &mut transactions);
        } else {
            self.match_sell_order(order, &mut transactions);
        }

        transactions
    }

    fn match_buy_order(&mut self, mut buy_order: Order, transactions: &mut Vec<Transaction>) {
        while let Some(best_ask) = self.order_book.get_best_ask() {
            if buy_order.price < best_ask.price || buy_order.quantity <= 0.0 {
                break;
            }

            let matched_quantity = buy_order.quantity.min(best_ask.quantity);
            let transaction = Transaction::new(
                best_ask.trader_id.clone(),
                buy_order.trader_id.clone(),
                matched_quantity,
                best_ask.price,
                buy_order.resource.clone(),
            );

            transactions.push(transaction);

            buy_order.quantity -= matched_quantity;
            let mut sell_order = self.order_book.remove_order(&best_ask.id, false).unwrap();
            sell_order.quantity -= matched_quantity;

            if sell_order.quantity > 0.0 {
                self.order_book.add_order(sell_order);
            }
        }

        if buy_order.quantity > 0.0 {
            self.order_book.add_order(buy_order);
        }
    }

    fn match_sell_order(&mut self, mut sell_order: Order, transactions: &mut Vec<Transaction>) {
        while let Some(best_bid) = self.order_book.get_best_bid() {
            if sell_order.price > best_bid.price || sell_order.quantity <= 0.0 {
                break;
            }

            let matched_quantity = sell_order.quantity.min(best_bid.quantity);
            let transaction = Transaction::new(
                sell_order.trader_id.clone(),
                best_bid.trader_id.clone(),
                matched_quantity,
                best_bid.price,
                sell_order.resource.clone(),
            );

            transactions.push(transaction);

            sell_order.quantity -= matched_quantity;
            let mut buy_order = self.order_book.remove_order(&best_bid.id, true).unwrap();
            buy_order.quantity -= matched_quantity;

            if buy_order.quantity > 0.0 {
                self.order_book.add_order(buy_order);
            }
        }

        if sell_order.quantity > 0.0 {
            self.order_book.add_order(sell_order);
        }
    }
}