use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;

pub struct Account {
    client_id: ClientId,
    available: Amount,
    held: Amount,
    locked: bool,
}
