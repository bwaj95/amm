use anchor_lang::{prelude::*, Key, ToAccountInfo};
use anchor_spl::token::{self, MintTo, TransferChecked};

use crate::{LP_MINT_SEED, POOL_SEED};

/// To use when a user signs a token transfer
pub fn transfer_tokens_checked<'info>(
    from: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    // CPI to TokenProgram::TransferChecked from user ata to another ata.
    // auto signed by user since user is the authority

    let cpi_program = token_program.to_account_info();

    let cpi_accounts = TransferChecked {
        from: from.to_account_info(),
        authority: authority.to_account_info(),
        to: to.to_account_info(),
        mint: mint.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program.key(), cpi_accounts);

    // call the transfer CPI
    token::transfer_checked(cpi_ctx, amount, decimals)?;

    Ok(())
}

/// To use when minting tokens from any mint to respective user ata
pub fn mint_tokens<'info>(
    mint: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    token_program: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    // CPI to TokenProgram::MintTo to mint token to user ata.
    // signer is mint authority, seeds and bump needed - for what ??

    let cpi_program = token_program.key();

    let cpi_accounts = MintTo {
        mint: mint.to_account_info(),
        authority: authority.to_account_info(),
        to: to.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);

    token::mint_to(cpi_ctx, amount)?;

    Ok(())
}
