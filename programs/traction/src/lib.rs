//! Program for issuing American options.

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate_token::CrateToken;
use vipers::*;

mod events;
mod ixs;
mod macros;
mod state;

pub use events::*;
pub use state::*;

/// Owner of all accounts that receives fees earned by the protocol.
/// This is a PDA.
pub static FEE_OWNER: Pubkey =
    static_pubkey::static_pubkey!("2DDSpDyRbu9gZbcp2JCq2ZaA9FrCzXzoiyiGLyUFYSP5");

/// Bump seed.
pub const FEE_OWNER_BUMP: u8 = 255;

/// Thousands of BPS of the exercise fee.
pub const EXERCISE_FEE_KBPS: u64 = 1_000;

declare_id!("TRXf3r361YRfV6Zktov3nvdEqJwAuCowkjh4PUUBYEc");

/// Traction program.
#[program]
pub mod traction {
    use super::*;

    /// Defines a new [OptionsContract].
    ///
    /// An [OptionsContract] is uniquely defined by four parameters:
    /// - `underlying_mint`, the mint of the underlying token
    /// - `quote_mint`, the mint of the quote token
    /// - `strike`, the strike price to purchase 10**underlying_decimals of the underlying
    /// - `expiry_ts`, when the option expires.
    ///
    /// Anyone can create the [OptionsContract].
    ///
    /// All [OptionsContract]s are call options on the underlying. To write a put option,
    /// one should invert the quote and underlying.
    #[access_control(ctx.accounts.validate())]
    pub fn new_contract(
        ctx: Context<NewContract>,
        strike: u64,
        expiry_ts: i64,
        is_put: bool,
        contract_bump: u8,
        crate_bump: u8,
    ) -> ProgramResult {
        ctx.accounts
            .new_contract(strike, expiry_ts, is_put, contract_bump, crate_bump)
    }

    #[access_control(ctx.accounts.validate())]
    pub fn option_burn(ctx: Context<OptionBurn>, write_amount: u64) -> ProgramResult {
        ctx.accounts.burn(write_amount)
    }

    /// Write new options
    #[access_control(ctx.accounts.validate())]
    pub fn option_write(ctx: Context<OptionWrite>, write_amount: u64) -> ProgramResult {
        ctx.accounts.write(write_amount)
    }

    /// Exercise an option
    #[access_control(ctx.accounts.validate())]
    pub fn option_exercise(ctx: Context<OptionExercise>, option_amount: u64) -> ProgramResult {
        ctx.accounts.exercise(option_amount)
    }

    /// Redeem `writer_mint` for the underlying collateral/exercise proceeds.
    #[access_control(ctx.accounts.validate())]
    pub fn option_redeem(ctx: Context<OptionRedeem>, writer_amount: u64) -> ProgramResult {
        ctx.accounts.redeem(writer_amount)
    }
}

/// Accounts for [traction::new_contract].
#[derive(Accounts)]
#[instruction(strike: u64, expiry_ts: u64, is_put: bool, contract_bump: u8)]
pub struct NewContract<'info> {
    #[account(
        init,
        seeds = [
            b"OptionsContract" as &[u8],
            underlying_mint.key().to_bytes().as_ref(),
            quote_mint.key().to_bytes().as_ref(),
            strike.to_le_bytes().as_ref(),
            expiry_ts.to_le_bytes().as_ref(),
            (if is_put { &[1_u8] } else { &[0_u8] }),
        ],
        bump = contract_bump,
        payer = payer
    )]
    pub contract: Account<'info, OptionsContract>,

    /// [Mint] of the underlying asset.
    pub underlying_mint: Account<'info, Mint>,
    /// [Mint] of the quote asset.
    pub quote_mint: Account<'info, Mint>,
    /// The [CrateToken] of the writer.
    pub writer_crate: WriterCrate<'info>,
    /// The [Mint] of the option instrument.
    pub option_mint: Account<'info, Mint>,

    /// Payer to fund accounts.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// System program.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WriterCrate<'info> {
    /// [Mint] of the [crate_token::CrateToken].
    pub crate_mint: Account<'info, Mint>,

    /// The [crate_token::CrateToken] to be created.
    #[account(mut)]
    pub crate_token: UncheckedAccount<'info>,

    /// Crate token program.
    pub crate_token_program: Program<'info, crate_token::program::CrateToken>,
}

/// Accounts for [traction::option_write].
#[derive(Accounts)]
pub struct OptionWrite<'info> {
    /// The authority of the [user_underlying_funding_tokens] account.
    #[account(mut)]
    pub writer_authority: Signer<'info>,
    /// The options contract.
    pub contract: Box<Account<'info, OptionsContract>>,
    /// The user's underlying tokens used to fund writing the options.
    #[account(mut)]
    pub user_underlying_funding_tokens: Box<Account<'info, TokenAccount>>,
    /// The option token account to send to.
    #[account(mut)]
    pub option_token_destination: Box<Account<'info, TokenAccount>>,
    /// The [OptionsContract::writer_crate]'s underlying tokens which collateralize the options.
    #[account(mut)]
    pub crate_underlying_tokens: Box<Account<'info, TokenAccount>>,

    /// The writer token account to send to.
    #[account(mut)]
    pub writer_token_destination: Box<Account<'info, TokenAccount>>,
    /// The writer crate token.
    pub writer_crate_token: Box<Account<'info, CrateToken>>,
    /// The writer mint.
    #[account(mut)]
    pub writer_mint: Box<Account<'info, Mint>>,

    /// The option mint.
    #[account(mut)]
    pub option_mint: Box<Account<'info, Mint>>,

    /// Token program.
    pub token_program: Program<'info, Token>,
    /// Crate token program.
    pub crate_token_program: Program<'info, crate_token::program::CrateToken>,
}

/// Accounts for [traction::option_exercise].
#[derive(Accounts)]
pub struct OptionExercise<'info> {
    /// The authority of the [option_token_source] account.
    #[account(mut)]
    pub exerciser_authority: Signer<'info>,
    /// The options contract.
    pub contract: Box<Account<'info, OptionsContract>>,

    /// The [exerciser_authority]'s quote tokens used to pay for the exercise of the options.
    #[account(mut)]
    pub quote_token_source: Box<Account<'info, TokenAccount>>,

    /// The option mint.
    #[account(mut)]
    pub option_mint: Box<Account<'info, Mint>>,
    /// The [exerciser_authority]'s options tokens used to fund writing the options.
    #[account(mut)]
    pub option_token_source: Box<Account<'info, TokenAccount>>,

    /// The writer crate token.
    pub writer_crate_token: Box<Account<'info, CrateToken>>,
    /// The writer crate's underlying tokens which collateralize the options.
    #[account(mut)]
    pub crate_underlying_tokens: Box<Account<'info, TokenAccount>>,
    /// The writer crate's quote tokens which are obtained when options are exercised.
    #[account(mut)]
    pub crate_quote_tokens: Box<Account<'info, TokenAccount>>,
    /// The option token account to send to.
    #[account(mut)]
    pub underlying_token_destination: Box<Account<'info, TokenAccount>>,
    /// The token account to send the quote asset to for exercise fees.
    #[account(mut)]
    pub exercise_fee_quote_destination: Box<Account<'info, TokenAccount>>,

    /// Token program.
    pub token_program: Program<'info, Token>,
    /// Crate token program.
    pub crate_token_program: Program<'info, crate_token::program::CrateToken>,
}

/// Accounts for [traction::option_burn].
#[derive(Accounts)]

pub struct OptionBurn<'info> {
    /// The authority of the [self::writer_token_source] account.
    #[account(mut)]
    pub writer_authority: Signer<'info>,

    pub contract: Box<Account<'info, OptionsContract>>,

    /// The [Mint] of the writer mint.
    #[account(mut)]
    pub writer_mint: Box<Account<'info, Mint>>,

    /// The [Mint] of the option instrument.
    #[account(mut)]
    pub option_mint: Box<Account<'info, Mint>>,

    /// The [burner_authority]'s quote tokens used to pay for the exercise of the options.
    #[account(mut)]
    pub writer_token_source: Box<Account<'info, TokenAccount>>,

    /// The [burner_authority]'s quote tokens used to pay for the exercise of the options.
    #[account(mut)]
    pub option_token_source: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub crate_underlying_tokens: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub underlying_token_destination: Box<Account<'info, TokenAccount>>,

    /// [Mint] of the underlying asset.
    pub underlying_mint: Account<'info, Mint>,

    /// The writer crate token.
    pub writer_crate_token: Box<Account<'info, CrateToken>>,

    /// The [CrateToken] of the writer.
    pub writer_crate: WriterCrate<'info>,

    #[account(mut)]
    pub crate_quote_tokens: Box<Account<'info, TokenAccount>>,

    /// Token program.
    pub token_program: Program<'info, Token>,
    /// Crate token program.
    pub crate_token_program: Program<'info, crate_token::program::CrateToken>,
}

/// Accounts for [traction::option_redeem].
#[derive(Accounts)]
pub struct OptionRedeem<'info> {
    /// The authority of the [self::writer_token_source] account.
    #[account(mut)]
    pub writer_authority: Signer<'info>,
    /// The options contract.
    pub contract: Box<Account<'info, OptionsContract>>,

    /// The writer's writer token account.
    #[account(mut)]
    pub writer_token_source: Box<Account<'info, TokenAccount>>,
    /// The writer mint.
    #[account(mut)]
    pub writer_mint: Box<Account<'info, Mint>>,
    /// The underlying token account to send to.
    #[account(mut)]
    pub underlying_token_destination: Box<Account<'info, TokenAccount>>,
    /// The quote token account to send to.
    #[account(mut)]
    pub quote_token_destination: Box<Account<'info, TokenAccount>>,

    /// The writer crate token.
    pub writer_crate_token: Box<Account<'info, CrateToken>>,
    /// The [CrateToken]'s underlying tokens which collateralize the options.
    #[account(mut)]
    pub crate_underlying_tokens: Box<Account<'info, TokenAccount>>,
    /// The [CrateToken]'s quote tokens.
    #[account(mut)]
    pub crate_quote_tokens: Box<Account<'info, TokenAccount>>,

    /// Token program.
    pub token_program: Program<'info, Token>,
    /// Crate token program.
    pub crate_token_program: Program<'info, crate_token::program::CrateToken>,
}

/// Error codes.
#[error]
pub enum ErrorCode {
    #[msg("Unauthorized.")]
    Unauthorized,
    #[msg("Insufficient collateral to write options.")]
    InsufficientCollateral,
    #[msg("Options contract is expired.")]
    ContractExpired,
    #[msg("Cannot redeem until contract expiry.")]
    ContractNotYetExpired,

    #[msg(
        "A call option mint must have the same decimals as the underlying.",
        offset = 10
    )]
    CallDecimalMismatch,
    #[msg("A put option mint must have the same decimals as the quote.")]
    PutDecimalMismatch,
    #[msg("The underlying and quote mints should not match.")]
    UselessMints,
    #[msg("Option mint must have zero supply.")]
    OptionMintMustHaveZeroSupply,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fee_owner_address() {
        let (key, bump) = Pubkey::find_program_address(&[b"TractionDAOFees"], &crate::ID);
        assert_eq!(key, FEE_OWNER);
        assert_eq!(bump, FEE_OWNER_BUMP);
    }
}
