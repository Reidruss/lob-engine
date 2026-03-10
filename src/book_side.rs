use std::collections::BTreeMap;

use crate::order::{Order, Side};
use crate::price_level::PriceLevel;

pub struct BookSide {
    pub(crate) levels: BTreeMap<u64, PriceLevel>,
}

impl BookSide {
    pub fn new() -> Self {
        Self {
            levels: BTreeMap::new(),
        }
    }

    /// Get best price (highest for bids, lowest for asks)
    pub fn best_price(&self, side: Side) -> Option<u64> {
        match side {
            Side::Bid => self.levels.keys().rev().next().cloned(),
            Side::Ask => self.levels.keys().next().cloned(),
        }
    }

    pub fn insert(&mut self, order: Order) {
        let level = self
            .levels
            .entry(order.price)
            .or_insert_with(|| PriceLevel::new(order.price));
        level.add_order(order);
    }

    pub fn remove_level_if_empty(&mut self, price: u64) {
        if let Some(level) = self.levels.get(&price) {
            if level.orders.is_empty() {
                self.levels.remove(&price);
            }
        }
    }
}
