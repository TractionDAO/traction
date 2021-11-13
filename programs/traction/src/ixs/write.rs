//! Handles [crate::traction::option_write]

use crate::*;
use anchor_spl::token;

impl<'info> OptionWrite<'info> {
    pub fn write(&self, write_amount: u64) -> ProgramResult {
        let user_underlying_funding_tokens = &self.user_underlying_funding_tokens;
        require!(
            user_underlying_funding_tokens.amount >= write_amount,
            InsufficientCollateral
        );

        self.pull_payment(write_amount)?;
        self.issue_writer_tokens(write_amount)?;
        self.mint_options(write_amount)?;

        emit!(OptionWriteEvent {
            contract: self.contract.key(),
            writer: self.writer_authority.key(),
            write_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// transfer writer's tokens to the crate
    fn pull_payment(&self, write_amount: u64) -> ProgramResult {
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.user_underlying_funding_tokens.to_account_info(),
                    to: self.crate_underlying_tokens.to_account_info(),
                    authority: self.writer_authority.to_account_info(),
                },
            ),
            write_amount,
        )
    }

    /// issue the writer tokens
    fn issue_writer_tokens(&self, write_amount: u64) -> ProgramResult {
        let seeds: &[&[&[u8]]] = gen_contract_signer_seeds!(self.contract);
        crate_token::cpi::issue(
            CpiContext::new_with_signer(
                self.crate_token_program.to_account_info(),
                crate_token::cpi::accounts::Issue {
                    crate_token: self.writer_crate_token.to_account_info(),
                    crate_mint: self.writer_mint.to_account_info(),
                    issue_authority: self.contract.to_account_info(),
                    mint_destination: self.writer_token_destination.to_account_info(),

                    // there are no author/protocol withdraw fees, so we pass in garbage here
                    author_fee_destination: self.writer_token_destination.to_account_info(),
                    protocol_fee_destination: self.writer_token_destination.to_account_info(),

                    token_program: self.token_program.to_account_info(),
                },
                seeds,
            ),
            write_amount,
        )
    }

    /// mint `option_amount` options
    fn mint_options(&self, write_amount: u64) -> ProgramResult {
        let seeds: &[&[&[u8]]] = gen_contract_signer_seeds!(self.contract);
        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::MintTo {
                    mint: self.option_mint.to_account_info(),
                    to: self.option_token_destination.to_account_info(),
                    authority: self.contract.to_account_info(),
                },
                seeds,
            ),
            write_amount,
        )
    }
}

impl<'info> Validate<'info> for OptionWrite<'info> {
    fn validate(&self) -> ProgramResult {
        let now = Clock::get()?.unix_timestamp;
        require!(now < self.contract.expiry_ts, ContractExpired);

        assert_keys_eq!(
            self.writer_authority,
            self.user_underlying_funding_tokens.owner
        );
        // option_token_destination checks are redundant
        assert_keys_eq!(
            self.crate_underlying_tokens,
            self.contract.crate_underlying_tokens
        );
        // writer_token_destination checks not needed
        assert_keys_eq!(self.writer_crate_token, self.contract.writer_crate);
        assert_keys_eq!(self.writer_mint, self.contract.writer_mint);
        assert_keys_eq!(self.option_mint, self.contract.option_mint);

        Ok(())
    }
}
