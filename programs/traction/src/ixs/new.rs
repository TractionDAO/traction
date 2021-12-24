//! Handles [crate::traction::new_contract].

use vipers::{assert_keys_eq, assert_keys_neq};

use crate::*;

impl<'info> NewContract<'info> {
    /// Creates a new [OptionsContract].
    pub fn new_contract(
        &mut self,
        strike: u64,
        expiry_ts: i64,
        is_put: bool,
        contract_bump: u8,
        crate_bump: u8,
    ) -> ProgramResult {
        // initialize the writer crate
        // The writer crate holds all options.
        crate_token::cpi::new_crate(
            CpiContext::new(
                self.writer_crate.crate_token_program.to_account_info(),
                crate_token::cpi::accounts::NewCrate {
                    crate_mint: self.writer_crate.crate_mint.to_account_info(),
                    crate_token: self.writer_crate.crate_token.to_account_info(),
                    // the contract can issue more writer tokens
                    issue_authority: self.contract.to_account_info(),
                    // the contract can withdraw from the crate
                    withdraw_authority: self.contract.to_account_info(),

                    fee_to_setter: self.contract.to_account_info(),
                    fee_setter_authority: self.contract.to_account_info(),
                    author_fee_to: self.contract.to_account_info(),
                    payer: self.payer.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
            ),
            crate_bump,
        )?;

        let contract = &mut self.contract;

        contract.underlying_mint = self.underlying_mint.key();
        contract.quote_mint = self.quote_mint.key();
        contract.strike = strike;
        contract.expiry_ts = expiry_ts;
        contract.is_put = is_put;
        contract.bump = contract_bump;

        contract.writer_mint = self.writer_crate.crate_mint.key();
        contract.writer_crate = self.writer_crate.crate_token.key();
        contract.crate_collateral_tokens =
            spl_associated_token_account::get_associated_token_address(
                &self.writer_crate.crate_token.key(),
                &contract.collateral_mint(),
            );
        contract.crate_exercise_tokens = spl_associated_token_account::get_associated_token_address(
            &self.writer_crate.crate_token.key(),
            &contract.exercise_mint(),
        );
        contract.option_mint = self.option_mint.key();

        Ok(())
    }
}

impl<'info> Validate<'info> for NewContract<'info> {
    fn validate(&self) -> ProgramResult {
        // crate accounts are checked by Crate Protocol.

        // ensure we have full control over the mint provided
        assert_keys_eq!(self.option_mint.mint_authority.unwrap(), self.contract);
        assert_keys_eq!(self.option_mint.freeze_authority.unwrap(), self.contract);
        invariant!(self.option_mint.supply == 0, OptionMintMustHaveZeroSupply);

        assert_keys_neq!(self.underlying_mint, self.quote_mint, UselessMints);

        invariant!(
            self.underlying_mint.decimals == self.option_mint.decimals,
            OptionDecimalMismatch
        );
        invariant!(
            self.underlying_mint.decimals == self.writer_crate.crate_mint.decimals,
            WriterDecimalMismatch
        );

        Ok(())
    }
}
