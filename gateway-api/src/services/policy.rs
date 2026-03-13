/// Transfer policy engine.
/// Evaluates whether a transfer is allowed by policy rules.
/// Validation (amount > 0, wallets exist, sufficient balance, etc.) is separate.

const MAX_TRANSFER_AMOUNT: i64 = 300;

/// Returns Ok(()) if the transfer is allowed by policy, Err(reason) if rejected.
pub fn evaluate_transfer(amount: i64) -> Result<(), &'static str> {
    if amount > MAX_TRANSFER_AMOUNT {
        Err("transfer amount exceeds policy limit")
    } else {
        Ok(())
    }
}
