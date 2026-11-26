//! Shared contracts. Anything public here is a wire/API promise to every consumer.
use serde::{Deserialize, Serialize};
use std::fmt;

/// HTTP header every mutating call MUST carry.
pub const IDEMPOTENCY_HEADER: &str = "Idempotency-Key";
/// Hard ceiling for a single charge, in minor units ($1,000,000.00).
pub const MAX_CHARGE_MINOR: i64 = 100_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    /// Zero-decimal currency: minor unit == major unit.
    JPY,
}

impl Currency {
    pub fn minor_unit_exponent(&self) -> u32 {
        match self {
            Currency::JPY => 0,
            _ => 2,
        }
    }
}

/// Money is ALWAYS integer minor units. Never floats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub amount_cents: i64,
    pub currency: Currency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChargeRequest {
    /// Required. Replays with the same key must return the original result.
    pub idempotency_key: String,
    pub customer_id: String,
    pub amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    MissingIdempotencyKey,
    NonPositiveAmount,
    AmountTooLarge,
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::MissingIdempotencyKey => write!(f, "idempotency key is required"),
            ContractError::NonPositiveAmount => write!(f, "amount must be positive"),
            ContractError::AmountTooLarge => write!(f, "amount exceeds MAX_CHARGE_MINOR"),
        }
    }
}

impl std::error::Error for ContractError {}

pub fn validate_charge(req: &ChargeRequest) -> Result<(), ContractError> {
    if req.idempotency_key.trim().is_empty() {
        return Err(ContractError::MissingIdempotencyKey);
    }
    if req.amount.amount_cents <= 0 {
        return Err(ContractError::NonPositiveAmount);
    }
    if req.amount.amount_cents > MAX_CHARGE_MINOR {
        return Err(ContractError::AmountTooLarge);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(key: &str, amt: i64) -> ChargeRequest {
        ChargeRequest {
            idempotency_key: key.into(),
            customer_id: "cus_1".into(),
            amount: Money { amount_cents: amt, currency: Currency::USD },
        }
    }

    #[test]
    fn rejects_missing_key() {
        assert_eq!(validate_charge(&req("", 100)), Err(ContractError::MissingIdempotencyKey));
    }

    #[test]
    fn accepts_valid() {
        assert!(validate_charge(&req("k1", 100)).is_ok());
    }
}
