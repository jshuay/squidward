use rust_decimal::Decimal;

use crate::payment_system::error::PaymentSystemError;

pub type Result<T> = std::result::Result<T, PaymentSystemError>;

pub type TransactionId = u32;

pub type ClientId = u16;

pub type Amount = Decimal;
