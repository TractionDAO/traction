//! Handles [crate::traction::option_redeem].

use crate::*;
use anchor_spl::token;
use num_traits::ToPrimitive;

impl<'info> OptionRedeem<'info> {
    /// Helper to redeem the writer crate.
    /// This is not necessary.
    pub fn redeem(&self, writer_amount: u64) -> ProgramResult {
        // calculate share of the tokens
        let collateral_amount = unwrap_int!((self.crate_collateral_tokens.amount as u128)
            .checked_mul(self.writer_token_source.amount.into())
            .and_then(|v| v.checked_div(self.writer_mint.supply.into()))
            .and_then(|v| v.to_u64()));

        // burn writer tokens
        token::burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Burn {
                    mint: self.writer_mint.to_account_info(),
                    to: self.writer_token_source.to_account_info(),
                    authority: self.writer_authority.to_account_info(),
                },
            ),
            writer_amount,
        )?;

        // withdraw proportional amounts of crate tokens
        let seeds: &[&[&[u8]]] = gen_contract_signer_seeds!(self.contract);
        crate_token::cpi::withdraw(
            CpiContext::new_with_signer(
                self.crate_token_program.to_account_info(),
                crate_token::cpi::accounts::Withdraw {
                    crate_token: self.writer_crate_token.to_account_info(),
                    crate_underlying: self.crate_collateral_tokens.to_account_info(),
                    withdraw_authority: self.contract.to_account_info(),
                    withdraw_destination: self.underlying_token_destination.to_account_info(),
                    // no fees here
                    author_fee_destination: self.underlying_token_destination.to_account_info(),
                    protocol_fee_destination: self.underlying_token_destination.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                },
                seeds,
            ),
            collateral_amount,
        )?;

        // redeem exercise tokens if they are different from the collateral tokens
        if self.crate_collateral_tokens.mint != self.crate_exercise_tokens.mint {
            let exercise_amount = unwrap_int!((self.crate_exercise_tokens.amount as u128)
                .checked_mul(self.writer_token_source.amount.into())
                .and_then(|v| v.checked_div(self.writer_mint.supply.into()))
                .and_then(|v| v.to_u64()));
            crate_token::cpi::withdraw(
                CpiContext::new_with_signer(
                    self.crate_token_program.to_account_info(),
                    crate_token::cpi::accounts::Withdraw {
                        crate_token: self.writer_crate_token.to_account_info(),
                        crate_underlying: self.crate_exercise_tokens.to_account_info(),
                        withdraw_authority: self.contract.to_account_info(),
                        withdraw_destination: self.quote_token_destination.to_account_info(),
                        // no fees here
                        author_fee_destination: self.quote_token_destination.to_account_info(),
                        protocol_fee_destination: self.quote_token_destination.to_account_info(),
                        token_program: self.token_program.to_account_info(),
                    },
                    seeds,
                ),
                exercise_amount,
            )?;
        }

        emit!(OptionRedeemEvent {
            contract: self.contract.key(),
            redeemer: self.writer_authority.key(),
            writer_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for OptionRedeem<'info> {
    fn validate(&self) -> ProgramResult {
        // can only redeem when the contract has expired.
        let now = Clock::get()?.unix_timestamp;
        invariant!(now >= self.contract.expiry_ts, ContractNotYetExpired);

        assert_keys_eq!(self.writer_authority, self.writer_token_source.owner);
        assert_keys_eq!(self.writer_token_source.mint, self.contract.writer_mint);
        // underlying_token_destination and quote_token_destination don't really matter to validate

        assert_keys_eq!(self.writer_crate_token, self.contract.writer_crate);
        assert_keys_eq!(
            self.crate_collateral_tokens,
            self.contract.crate_collateral_tokens
        );
        assert_keys_eq!(
            self.crate_exercise_tokens,
            self.contract.crate_exercise_tokens
        );

        Ok(())
    }
}
