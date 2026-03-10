use crate::book_side::BookSide;
use crate::order::{Order, Side};

pub struct OrderBook {
    bids: BookSide,
    asks: BookSide,
    next_order_id: u64,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BookSide::new(),
            asks: BookSide::new(),
            next_order_id: 1,
        }
    }

    pub fn submit_limit_order(&mut self, mut order: Order) -> u64 {
        let (own_side, other_side) = match order.side {
            Side::Bid => (&mut self.bids, &mut self.asks),
            Side::Ask => (&mut self.asks, &mut self.bids),
        };

        while order.original_quantity > 0 {
            // Peek best opposite price
            if let Some(best_price) = other_side.best_price(match order.side {
                Side::Bid => Side::Ask,
                Side::Ask => Side::Bid,
            }) {
                let should_match = match order.side {
                    Side::Bid => order.price >= best_price,
                    Side::Ask => order.price <= best_price,
                };
                if !should_match {
                    break;
                }

                // Match at this price level
                if let Some(level) = other_side.levels.get_mut(&best_price) {
                    while let Some(mut resting) = level.pop_front() {
                        let traded = resting.original_quantity.min(order.original_quantity);
                        resting.original_quantity -= traded;
                        order.original_quantity -= traded;

                        if resting.original_quantity > 0 {
                            level.orders.push_front(resting);
                            break;
                        }
                        if order.original_quantity == 0 {
                            break;
                        }
                    }
                    other_side.remove_level_if_empty(best_price);
                }
            } else {
                break;
            }
        }

        // If there is remaining quantity, insert into own side
        if order.original_quantity > 0 {
            own_side.insert(order);
        }

        order.original_quantity
    }
}
