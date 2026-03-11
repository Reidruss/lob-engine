pub type OrderId = u64;
pub type Price = u64;
pub type Quantity = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy)]
pub struct Order {
    pub order_id: OrderId,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub original_quantity: Quantity,
    pub remaining_quantity: Quantity,
    pub timestamp: u64,
}
