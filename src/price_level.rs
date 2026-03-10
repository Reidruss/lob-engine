use std::collections::VecDeque;

use crate::order::Order;

/// Orders at a single price level
#[derive(Debug)]
pub struct PriceLevel {
    pub price: u64,
    pub orders: VecDeque<Order>,
}

impl PriceLevel {
    pub fn new(price: u64) -> Self {
        Self {
            price,
            orders: VecDeque::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        self.orders.push_back(order);
    }

    pub fn pop_front(&mut self) -> Option<Order> {
        self.orders.pop_front()
    }

    pub fn peek_front(&self) -> Option<&Order> {
        self.orders.front()
    }
}
