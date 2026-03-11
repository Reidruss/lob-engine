use crate::order::{OrderId, Price, Quantity, Side};

#[derive(Debug, Clone)]
pub struct StopOrder {
    pub id: OrderId,
    pub side: Side,
    pub trigger_price: Price,
    pub quantity: Quantity,
}
