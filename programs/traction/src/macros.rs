//! Macros.

#[macro_export]
macro_rules! gen_contract_signer_seeds {
    ($contract:expr) => {
        &[&[
            b"OptionsContract" as &[u8],
            &$contract.underlying_mint.to_bytes(),
            &$contract.quote_mint.to_bytes(),
            &$contract.strike.to_le_bytes(),
            &$contract.expiry_ts.to_le_bytes(),
            (if $contract.is_put { &[1_u8] } else { &[0_u8] }),
            &[$contract.bump],
        ]]
    };
}
