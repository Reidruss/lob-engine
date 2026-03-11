use crate::order::{Order, OrderId, Quantity};

#[derive(Debug, Clone)]
pub enum OrderEvent {
    OrderPlaced(Order),
    OrderPartiallyFilled { id: OrderId, filled: Quantity },
    OrderFullyFilled(OrderId),
    OrderCancelled(OrderId),
    IcebergRevealed(OrderId, Quantity),
    StopTriggered(OrderId),
}

pub struct EventStore {
    pub events: Vec<OrderEvent>,
}

impl EventStore {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn append(&mut self, event: OrderEvent) {
        self.events.push(event);
    }
}
