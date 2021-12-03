//! Handles [crate::traction::option_exercise].

use crate::*;
use anchor_spl::token;

impl<'info> OptionExercise<'info> {
    /// Exercise the option
    pub fn exercise(&self, option_amount: u64) -> ProgramResult {
        let contract = &self.contract;
        let exercise_amount: u64 =
            unwrap_int!(contract.calculate_exercise_amount_for_options(option_amount));

        // Send exercise tokens from exerciser to the writer crate
        let exercise_fee = unwrap_int!(exercise_amount
            .checked_mul(EXERCISE_FEE_KBPS)
            .and_then(|f| f.checked_div(10_000 * 1_000)));
        let exercise_received = unwrap_int!(exercise_amount.checked_sub(exercise_fee));

        // exercise quote
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.exercise_token_source.to_account_info(),
                    to: self.crate_exercise_tokens.to_account_info(),
                    authority: self.exerciser_authority.to_account_info(),
                },
            ),
            exercise_received,
        )?;
        // exercise fee
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.exercise_token_source.to_account_info(),
                    to: self.exercise_fee_destination.to_account_info(),
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

        // Send collateral tokens from crate to user
        let seeds: &[&[&[u8]]] = gen_contract_signer_seeds!(self.contract);
        crate_token::cpi::withdraw(
            CpiContext::new_with_signer(
                self.crate_token_program.to_account_info(),
                crate_token::cpi::accounts::Withdraw {
                    crate_token: self.writer_crate_token.to_account_info(),
                    crate_underlying: self.crate_collateral_tokens.to_account_info(),
                    withdraw_authority: self.contract.to_account_info(),
                    withdraw_destination: self.collateral_token_destination.to_account_info(),
                    // no fees here
                    author_fee_destination: self.collateral_token_destination.to_account_info(),
                    protocol_fee_destination: self.collateral_token_destination.to_account_info(),
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

        assert_keys_eq!(self.exercise_token_source.owner, self.exerciser_authority);
        assert_keys_eq!(
            self.exercise_token_source.mint,
            self.contract.exercise_mint()
        );
        assert_keys_eq!(self.option_mint, self.contract.option_mint);
        assert_keys_eq!(self.option_token_source.owner, self.exerciser_authority);
        assert_keys_eq!(self.option_token_source.mint, self.contract.option_mint);
        assert_keys_eq!(self.writer_crate_token, self.contract.writer_crate);
        assert_keys_eq!(
            self.crate_collateral_tokens,
            self.contract.crate_collateral_tokens
        );
        assert_keys_eq!(
            self.crate_exercise_tokens,
            self.contract.crate_exercise_tokens
        );
        assert_keys_eq!(
            self.collateral_token_destination.mint,
            self.contract.underlying_mint
        );

        assert_keys_eq!(self.exercise_fee_destination.owner, FEE_OWNER);

        Ok(())
    }
}
