//! Handles [crate::traction::option_redeem].

use crate::*;
use anchor_spl::token;

impl<'info> OptionBurn<'info> {
    /// Helper to burn options in case one no longer wants exposure.
    pub fn burn(&self, burn_amount: u64) -> ProgramResult {
        // exit if nothing to burn
        if burn_amount == 0 {
            return Ok(());
        }

        invariant!(
            burn_amount <= self.writer_token_source.amount,
            "insufficient funds"
        );
        invariant!(
            burn_amount <= self.option_token_source.amount,
            "insufficient funds"
        );

        // burn writer tokens
        token::burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Burn {
                    mint: self.writer_mint.to_account_info(),
                    to: self.writer_token_source.to_account_info(),
                    authority: self.burner_authority.to_account_info(),
                },
            ),
            burn_amount,
        )?;

        // burn option tokens
        token::burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Burn {
                    mint: self.option_mint.to_account_info(),
                    to: self.option_token_source.to_account_info(),
                    authority: self.burner_authority.to_account_info(),
                },
            ),
            burn_amount,
        )?;

        // Send underlying tokens from crate to user
        let seeds: &[&[&[u8]]] = gen_contract_signer_seeds!(self.contract);
        crate_token::cpi::withdraw(
            CpiContext::new_with_signer(
                self.crate_token_program.to_account_info(),
                crate_token::cpi::accounts::Withdraw {
                    crate_token: self.writer_crate_token.to_account_info(),
                    crate_underlying: self.crate_underlying_tokens.to_account_info(),
                    withdraw_authority: self.contract.to_account_info(),
                    withdraw_destination: self.underlying_token_destination.to_account_info(),

                    // no fees here
                    author_fee_destination: self.underlying_token_destination.to_account_info(),
                    protocol_fee_destination: self.underlying_token_destination.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                },
                seeds,
            ),
            burn_amount,
        )?;

        emit!(OptionBurnEvent {
            contract: self.contract.key(),
            burn_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for OptionBurn<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.option_mint, self.contract.option_mint);
        assert_keys_eq!(self.writer_mint, self.contract.writer_mint);

        assert_keys_eq!(self.writer_token_source.mint, self.contract.writer_mint);
        assert_keys_eq!(self.writer_token_source.owner, self.burner_authority);

        assert_keys_eq!(self.option_token_source.mint, self.contract.option_mint);
        assert_keys_eq!(self.option_token_source.owner, self.burner_authority);

        // underlying_token_destination and quote_token_destination don't really matter to validate
        assert_keys_eq!(self.writer_crate_token, self.contract.writer_crate);
        assert_keys_eq!(
            self.crate_underlying_tokens,
            self.contract.crate_underlying_tokens
        );

        Ok(())
    }
}
