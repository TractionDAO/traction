//! Handles [crate::traction::option_exercise].

use crate::*;
use anchor_spl::token;

impl<'info> OptionExercise<'info> {
    /// Exercise the option
    pub fn exercise(&self, option_amount: u64) -> ProgramResult {
        let contract = &self.contract;
        let quote_amount: u64 =
            unwrap_int!(contract.calculate_quote_amount_for_options(option_amount));

        // Send quote tokens from exerciser to the writer crate
        let exercise_fee = unwrap_int!(quote_amount
            .checked_mul(EXERCISE_FEE_KBPS)
            .and_then(|f| f.checked_div(10_000 * 1_000)));
        let quote_received = unwrap_int!(quote_amount.checked_sub(exercise_fee));

        // exercise quote
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.quote_token_source.to_account_info(),
                    to: self.crate_quote_tokens.to_account_info(),
                    authority: self.exerciser_authority.to_account_info(),
                },
            ),
            quote_received,
        )?;
        // exercise fee
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.quote_token_source.to_account_info(),
                    to: self.exercise_fee_quote_destination.to_account_info(),
                    authority: self.exerciser_authority.to_account_info(),
                },
            ),
            exercise_fee,
        )?;

        // Burn exerciser's option tokens
        token::burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Burn {
                    mint: self.option_mint.to_account_info(),
                    to: self.option_token_source.to_account_info(),
                    authority: self.exerciser_authority.to_account_info(),
                },
            ),
            option_amount,
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
            option_amount,
        )?;

        emit!(OptionExerciseEvent {
            contract: self.contract.key(),
            exerciser: self.exerciser_authority.key(),
            option_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

impl<'info> Validate<'info> for OptionExercise<'info> {
    fn validate(&self) -> ProgramResult {
        let now = Clock::get()?.unix_timestamp;
        require!(now < self.contract.expiry_ts, ContractExpired);

        assert_keys_eq!(self.quote_token_source.owner, self.exerciser_authority);
        assert_keys_eq!(self.quote_token_source.mint, self.contract.quote_mint);
        assert_keys_eq!(self.option_mint, self.contract.option_mint);
        assert_keys_eq!(self.option_token_source.owner, self.exerciser_authority);
        assert_keys_eq!(self.option_token_source.mint, self.contract.option_mint);
        assert_keys_eq!(self.writer_crate_token, self.contract.writer_crate);
        assert_keys_eq!(
            self.crate_underlying_tokens,
            self.contract.crate_underlying_tokens
        );
        assert_keys_eq!(self.crate_quote_tokens, self.contract.crate_quote_tokens);
        assert_keys_eq!(
            self.underlying_token_destination.mint,
            self.contract.underlying_mint
        );

        Ok(())
    }
}
