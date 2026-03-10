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
    pub order_id: u64,
    pub side: Side,
    pub order_type: OrderType,
    pub price: u64,
    pub original_quantity: u64,
    pub remaining_quantity: u64,
    pub timestamp: u64,
}
