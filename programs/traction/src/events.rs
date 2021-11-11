//! Events.

use crate::*;

/// Emitted on [traction::option_write].
#[event]
pub struct OptionWriteEvent {
    /// The [OptionsContract].
    pub contract: Pubkey,
    /// The writer of the options.
    pub writer: Pubkey,
    /// The amount of options written.
    pub write_amount: u64,
    /// Timestamp of the event.
    pub timestamp: i64,
}

/// Emitted on [traction::option_redeem].
#[event]
pub struct OptionRedeemEvent {
    /// The [OptionsContract].
    pub contract: Pubkey,
    /// The redeemer of the writer tokens.
    pub redeemer: Pubkey,
    /// The amount of writer tokens redeemed.
    pub writer_amount: u64,
    /// Timestamp of the event.
    pub timestamp: i64,
}

/// Emitted on [traction::option_exercise].
#[event]
pub struct OptionExerciseEvent {
    /// The [OptionsContract].
    pub contract: Pubkey,
    /// The account that exercised the options.
    pub exerciser: Pubkey,
    /// The amount of options exercised.
    pub option_amount: u64,
    /// Timestamp of the event.
    pub timestamp: i64,
}
