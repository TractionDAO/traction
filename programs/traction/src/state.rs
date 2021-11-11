//! State accounts.

use crate::*;

/// Number of units of the underlying the strike price is denominated in.
pub const STRIKE_PRICE_UNITS: u64 = 1_000_000_000;

/// American option
#[account]
#[derive(Default)]
pub struct OptionsContract {
    /// Underlying asset
    pub underlying_mint: Pubkey,
    /// Strike price is denominated in this
    pub quote_mint: Pubkey,
    /// Number of `quote_mint` tokens required to purchase `10^9` of the `underlying_mint`.
    pub strike: u64,
    /// When the option expires.
    pub expiry_ts: i64,
    /// If the Option prefers to be rendered as a PUT.
    /// If true, the decimals of the token must be equal to that of the quote mint.
    /// If false, the decimals of the options token should be equal to that of the underlying mint.
    pub is_put: bool,
    /// Bump seed.
    pub bump: u8,

    /// The right to receive the proceeds from the option being exercised.
    pub writer_mint: Pubkey,
    /// The [crate_token::CrateToken] of the writer tokens.
    pub writer_crate: Pubkey,
    /// The underlying tokens of the crate.
    pub crate_underlying_tokens: Pubkey,
    /// The quote tokens of the crate.
    pub crate_quote_tokens: Pubkey,
    /// The option which can be exercised.
    pub option_mint: Pubkey,
}

impl OptionsContract {
    pub fn calculate_quote_amount_for_options(&self, option_amount: u64) -> Option<u64> {
        option_amount
            .checked_mul(self.strike)?
            .checked_div(STRIKE_PRICE_UNITS)
    }
}
