//! State accounts.

use num_traits::cast::ToPrimitive;

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
    /// If the option is a put.
    pub is_put: bool,
    /// Bump seed.
    pub bump: u8,

    /// The right to receive the proceeds from the option being exercised.
    pub writer_mint: Pubkey,
    /// The [crate_token::CrateToken] of the writer tokens.
    pub writer_crate: Pubkey,
    /// The collateral tokens of the crate.
    pub crate_collateral_tokens: Pubkey,
    /// The exercise tokens of the crate.
    pub crate_exercise_tokens: Pubkey,
    /// The option which can be exercised.
    pub option_mint: Pubkey,
}

impl OptionsContract {
    /// Mint of the collateral.
    /// If a call, this is the underlying.
    /// If a put, this is the quote.
    pub fn collateral_mint(&self) -> Pubkey {
        if self.is_put {
            self.quote_mint
        } else {
            self.underlying_mint
        }
    }

    /// Mint of the exercise.
    pub fn exercise_mint(&self) -> Pubkey {
        if self.is_put {
            self.underlying_mint
        } else {
            self.quote_mint
        }
    }

    /// Calculates the number of exercise tokens that correspond
    /// to the number of options tokens.
    /// The amount is equal
    pub fn calculate_exercise_amount_for_options(&self, option_amount: u64) -> Option<u64> {
        if self.is_put {
            // if underlying is the token to pay the exercise with,
            // divide by the strike price to get the exercise amount.
            (option_amount as u128)
                .checked_mul(STRIKE_PRICE_UNITS.into())?
                .checked_div(self.strike.into())?
                .to_u64()
        } else {
            (option_amount as u128)
                .checked_mul(self.strike.into())?
                .checked_div(STRIKE_PRICE_UNITS.into())?
                .to_u64()
        }
    }
}
