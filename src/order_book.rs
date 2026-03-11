use std::collections::HashMap;

use crate::book_side::BookSide;
use crate::event::{EventStore, OrderEvent};
use crate::iceberg::IcebergOrder;
use crate::order::{Order, OrderId, OrderType, Price, Quantity, Side};
use crate::stop_order::StopOrder;

pub struct OrderBook {
    bids: BookSide,
    asks: BookSide,
    next_order_id: OrderId,
    pub event_store: EventStore,
    iceberg_meta: HashMap<OrderId, IcebergOrder>,
    stop_orders: Vec<StopOrder>,
    last_trade_price: Option<Price>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BookSide::new(),
            asks: BookSide::new(),
            next_order_id: 1,
            event_store: EventStore::new(),
            iceberg_meta: HashMap::new(),
            stop_orders: Vec::new(),
            last_trade_price: None,
        }
    }

    fn assign_id(&mut self) -> OrderId {
        let id = self.next_order_id;
        self.next_order_id += 1;
        id
    }

    pub fn submit_limit_order(&mut self, mut order: Order) -> Quantity {
        order.order_id = self.assign_id();
        self.event_store
            .append(OrderEvent::OrderPlaced(order.clone()));
        let remaining = self.match_order(order);
        self.check_stop_orders();
        remaining
    }

    pub fn submit_iceberg_order(&mut self, mut iceberg: IcebergOrder) -> Quantity {
        let id = self.assign_id();
        iceberg.id = id;

        let visible = iceberg.visible_quantity.min(iceberg.remaining);
        iceberg.remaining -= visible;
        self.iceberg_meta.insert(id, iceberg.clone());

        let order = Order::new(id, iceberg.side, OrderType::Limit, iceberg.price, visible);

        self.event_store
            .append(OrderEvent::OrderPlaced(order.clone()));
        let remaining = self.match_order(order);
        self.check_stop_orders();
        remaining
    }

    pub fn add_stop_order(&mut self, mut stop: StopOrder) {
        stop.id = self.assign_id();
        self.stop_orders.push(stop);
    }

    fn match_order(&mut self, order: Order) -> Quantity {
        let initial_quantity = order.original_quantity;
        let event_store = &mut self.event_store;
        let iceberg_meta = &mut self.iceberg_meta;
        let last_trade = &mut self.last_trade_price;
        let mut taker_remaining = order.get_remaining_quantity();

        let (own_side, other_side) = match order.side {
            Side::Bid => (&mut self.bids, &mut self.asks),
            Side::Ask => (&mut self.asks, &mut self.bids),
        };

        let opposite = match order.side {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        };

        while taker_remaining > 0 {
            if let Some(best_price) = other_side.best_price(opposite) {
                let should_match = match order.side {
                    Side::Bid => order.price >= best_price,
                    Side::Ask => order.price <= best_price,
                };
                if !should_match {
                    break;
                }

                if let Some(level) = other_side.levels.get_mut(&best_price) {
                    while let Some(resting) = level.pop_front() {
                        let resting_remaining = resting.get_remaining_quantity();
                        let traded = resting_remaining.min(taker_remaining);
                        let resting_filled = resting.fill(traded);
                        let _taker_filled = order.fill(resting_filled);
                        taker_remaining = order.get_remaining_quantity();

                        *last_trade = Some(best_price);

                        if resting.get_remaining_quantity() > 0 {
                            event_store.append(OrderEvent::OrderPartiallyFilled {
                                id: resting.order_id,
                                filled: resting_filled,
                            });
                            level.orders.push_front(resting);
                            break;
                        } else {
                            event_store.append(OrderEvent::OrderFullyFilled(resting.order_id));

                            // Iceberg refill: reveal next visible slice
                            if let Some(meta) = iceberg_meta.get_mut(&resting.order_id) {
                                if meta.remaining > 0 {
                                    let new_visible = meta.visible_quantity.min(meta.remaining);
                                    meta.remaining -= new_visible;

                                    let refill = Order::new(
                                        resting.order_id,
                                        resting.side,
                                        OrderType::Limit,
                                        resting.price,
                                        new_visible,
                                    );

                                    level.orders.push_back(refill);
                                    event_store.append(OrderEvent::IcebergRevealed(
                                        resting.order_id,
                                        new_visible,
                                    ));
                                }
                            }
                        }

                        if taker_remaining == 0 {
                            break;
                        }
                    }
                    other_side.remove_level_if_empty(best_price);
                }
            } else {
                break;
            }
        }

        let filled = initial_quantity - taker_remaining;

        if taker_remaining > 0 {
            if filled > 0 {
                event_store.append(OrderEvent::OrderPartiallyFilled {
                    id: order.order_id,
                    filled,
                });
            }
            // Only rest limit orders in the book; market orders don't rest
            if order.order_type == OrderType::Limit {
                own_side.insert(order);
            }
        } else {
            event_store.append(OrderEvent::OrderFullyFilled(order.order_id));
        }

        taker_remaining
    }

    fn check_stop_orders(&mut self) {
        loop {
            let price = match self.last_trade_price {
                Some(p) => p,
                None => return,
            };

            let mut triggered = Vec::new();
            self.stop_orders.retain(|stop| {
                let should_trigger = match stop.side {
                    // Buy stop: triggers when price rises to or above trigger
                    Side::Bid => price >= stop.trigger_price,
                    // Sell stop: triggers when price falls to or below trigger
                    Side::Ask => price <= stop.trigger_price,
                };
                if should_trigger {
                    triggered.push(stop.clone());
                    false
                } else {
                    true
                }
            });

            if triggered.is_empty() {
                break;
            }

            for stop in triggered {
                self.event_store.append(OrderEvent::StopTriggered(stop.id));
                let order = Order::new(
                    stop.id,
                    stop.side,
                    OrderType::Market,
                    match stop.side {
                        Side::Bid => Price::MAX,
                        Side::Ask => 0,
                    },
                    stop.quantity,
                );
                self.match_order(order);
            }
        }
    }
}
