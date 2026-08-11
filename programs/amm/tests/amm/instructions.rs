use ::amm as amm_program;
use amm::state::protocol_config;
use amm_program::{accounts, instruction};

use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use anchor_spl::{
    associated_token::{self, get_associated_token_address},
    token,
};
use solana_message::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    amm::pdas::{
        find_locked_lp_token_pda, find_lp_mint_pda, find_mint_pda, find_pool_pda,
        find_protocol_config_pda, find_protocol_treasury_pda,
    },
    common::context::TestContext,
};

pub fn initialize_protocol_ix(ctx: &TestContext) -> Instruction {
    let program_id = &ctx.program_id;

    let (protocol_config, _) = find_protocol_config_pda(program_id);
    let (protocol_treasury, _) = find_protocol_treasury_pda(program_id);

    let accounts = accounts::InitializeProtocol {
        admin: ctx.admin.pubkey(),
        protocol_config: protocol_config,
        protocol_treasury: protocol_treasury,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction::InitializeProtocol {
            swap_fee_bps: 30,
            treasury_fee_bps: 5,
        }
        .data(), // data not available in suggesstions. InstructionData import solved. why?
    };

    instruction
}

pub fn create_pool_ix(
    ctx: &TestContext,
    creator: &Pubkey,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
) -> Instruction {
    let program_id = &ctx.program_id;

    let (protocol_config, _) = find_protocol_config_pda(program_id);
    let (pool, _) = find_pool_pda(program_id, mint_a, mint_b);
    let (lp_mint, _) = find_lp_mint_pda(program_id, &pool);
    let (locked_lp_token, _) = find_locked_lp_token_pda(program_id, &pool);

    let vault_a = get_associated_token_address(&pool, mint_a);
    let vault_b = get_associated_token_address(&pool, mint_b);

    let accounts = accounts::CreatePool {
        creator: *creator,
        pool,
        protocol_config,
        mint_a: *mint_a,
        mint_b: *mint_b,
        vault_a,
        vault_b,
        lp_mint,
        locked_lp_token,
        system_program: system_program::ID,
        token_program: token::ID,
        associated_token_program: associated_token::ID,
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction::CreatePool {}.data(),
    };

    instruction
}

pub fn initialize_mint_ix(
    ctx: &TestContext,
    admin: &Pubkey,
    mint_id: u64,
    decimals: u8,
) -> Instruction {
    let program_id = &ctx.program_id;

    let (protocol_config, _) = find_protocol_config_pda(program_id);
    let (mint_pda, _) = find_mint_pda(program_id, mint_id);

    let accounts = accounts::InitializeMint {
        admin: *admin,
        protocol_config,
        mint: mint_pda,
        system_program: system_program::ID,
        token_program: token::ID,
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction::InitializeMint { mint_id, decimals }.data(),
    };

    instruction
}

pub fn mint_tokens_ix(
    ctx: &TestContext,
    admin: &Pubkey,
    user: &Pubkey,
    user_ata: &Pubkey,
    mint_id: u64,
    amount: u64,
) -> Instruction {
    let program_id = ctx.program_id;

    let (protocol_config, _) = find_protocol_config_pda(&program_id);
    let (mint_pda, _) = find_mint_pda(&program_id, mint_id);

    // println!("[mint_tokens_ix] admin: {}", admin);
    // println!("[mint_tokens_ix] mint: {}", mint_pda);
    // println!("[mint_tokens_ix] protocol_config: {}", protocol_config);
    // println!("[mint_tokens_ix] user: {}", user);
    // println!("[mint_tokens_ix] user_ata: {}", user_ata);
    // println!("[mint_tokens_ix] system_program: {}", system_program::ID);
    // println!("[mint_tokens_ix] token_program: {}", token::ID);
    // println!("[mint_tokens_ix] associated_token_program: {}", associated_token::ID);

    let accounts = accounts::MintTokens {
        admin: *admin,
        mint: mint_pda,
        protocol_config,
        user: *user,
        user_ata: *user_ata,
        system_program: system_program::ID,
        token_program: token::ID,
        associated_token_program: associated_token::ID,
    }
    .to_account_metas(None);

    // for acc in &accounts {
    //     println!("[mint_tokens_ix] account: {} is_signer={} is_writable={}", acc.pubkey, acc.is_signer, acc.is_writable);
    // }

    let instruction = Instruction {
        program_id,
        accounts,
        data: instruction::MintTokens { mint_id, amount }.data(),
    };

    instruction
}

pub fn add_initial_liquidity_ix(
    ctx: &TestContext,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    provider: &Pubkey,
    amount_token_a: u64,
    amount_token_b: u64,
) -> Instruction {
    let program_id = ctx.program_id;

    let (pool, _) = find_pool_pda(&program_id, mint_a, mint_b);
    let vault_a = get_associated_token_address(&pool, mint_a);
    let vault_b = get_associated_token_address(&pool, mint_b);
    let (lp_mint, _) = find_lp_mint_pda(&program_id, &pool);
    let (locked_lp_token, _) = find_locked_lp_token_pda(&program_id, &pool);
    let provider_token_a_ata = get_associated_token_address(provider, mint_a);
    let provider_token_b_ata = get_associated_token_address(provider, mint_b);
    let provider_lp_token = get_associated_token_address(provider, &lp_mint);

    let accounts = accounts::AddInitialLiquidity {
        provider: *provider,
        pool,
        mint_a: *mint_a,
        mint_b: *mint_b,
        lp_mint,
        vault_a,
        vault_b,
        provider_token_a_ata,
        provider_token_b_ata,
        locked_lp_token,
        provider_lp_token,
        system_program: system_program::ID,
        token_program: token::ID,
        associated_token_program: associated_token::ID,
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id,
        accounts,
        data: instruction::AddInitialLiquidity {
            amount_token_a,
            amount_token_b,
        }
        .data(),
    };

    instruction
}
