use crate::order::{OrderId, Price, Quantity, Side};

#[derive(Debug, Clone)]
pub struct IcebergOrder {
    pub id: OrderId,
    pub side: Side,
    pub price: Price,
    pub total_quantity: Quantity,
    pub visible_quantity: Quantity,
    pub remaining: Quantity,
}
