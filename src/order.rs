use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Order {
    pub order_id: OrderId,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub original_quantity: Quantity,
    pub remaining_quantity: AtomicU64,
    pub status: AtomicU8,
    pub timestamp: u64,
}

impl Order {
    pub fn new(
        order_id: OrderId,
        side: Side,
        order_type: OrderType,
        price: Price,
        original_quantity: Quantity,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            order_id,
            side,
            order_type,
            price,
            original_quantity,
            remaining_quantity: AtomicU64::new(original_quantity),
            status: AtomicU8::new(OrderStatus::Active as u8),
            timestamp,
        }
    }

    pub fn get_remaining_quantity(&self) -> Quantity {
        self.remaining_quantity.load(Ordering::Acquire)
    }

    pub fn get_status(&self) -> OrderStatus {
        OrderStatus::from(self.status.load(Ordering::Acquire))
    }

    /// Fills up to `quantity` from this order atomically and returns the actual filled amount.
    pub fn fill(&self, quantity: Quantity) -> Quantity {
        let mut current = self.remaining_quantity.load(Ordering::Acquire);

        loop {
            if current == 0 {
                return 0;
            }

            let fill_amount = current.min(quantity);
            let new_remaining = current - fill_amount;

            match self.remaining_quantity.compare_exchange_weak(
                current,
                new_remaining,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    // Successflly updated remaining quantity
                    if new_remaining == 0 {
                        self.status
                            .store(OrderStatus::Filled as u8, Ordering::Release);
                    } else {
                        self.status
                            .store(OrderStatus::PartiallyFilled as u8, Ordering::Release)
                    }
                    return fill_amount;
                }
                Err(observed) => {
                    current = observed;
                }
            }
        }
    }

    pub fn cancel(&self) {
        let mut current_status = self.status.load(Ordering::Acquire);

        loop {
            if current_status == OrderStatus::Filled as u8
                || current_status == OrderStatus::Cancelled as u8
            {
                return false;
            }

            match self.status.compare_exchange_weak(
                current_status,
                OrderStatus::Cancelled as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return true;
                }
                Err(observed) => {
                    current_status = observed;
                }
            }
        }
    }

    pub fn is_active(&self) -> bool {
        let status = self.get_status();
        status == OrderStatus::Active || status == OrderStatus::PartiallyFilled
    }
}
