use anchor_spl::associated_token::get_associated_token_address;
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;

use crate::{
    amm::{
        accounts::{mint, pool, protocol_treasury, token_account}, add_initial_liquidity::add_initial_liquidity, add_liquidity::add_liquidity, create_pool::create_pool, initialize_mint::initialize_mint, initialize_protocol::initialize_protocol, mint_tokens::mint_tokens, pdas::{
            find_locked_lp_token_pda, find_lp_mint_pda, find_mint_pda, find_pool_pda,
            find_protocol_treasury_pda,
        }, remove_liquidity::remove_liquidity, structs::{InitializedPoolStruct, PoolReservesSnapshot, UserPoolAccounts}, swap::{self, swap}
    },
    common::context::TestContext,
};

pub fn setup_initialized_pool(
    ctx: &mut TestContext,
    alice_mint_amount: u64,
    initial_amount_a: u64,
    initial_amount_b: u64,
) -> Result<(InitializedPoolStruct, bool), FailedTransactionMetadata> {
    let program_id = ctx.program_id;
    let mut amount_a = initial_amount_a;
    let mut amount_b = initial_amount_b;

    // 1. Initilize Protocol
    initialize_protocol(ctx)?;

    // 2. Create Test Mints
    let mut mint_a_id = 1_u64;
    let mut mint_b_id = 2_u64;

    initialize_mint(ctx, mint_a_id, 9)?;
    initialize_mint(ctx, mint_b_id, 9)?;

    let (mut mint_a, _) = find_mint_pda(&program_id, mint_a_id);
    let (mut mint_b, _) = find_mint_pda(&program_id, mint_b_id);

    let mut mints_swapped = false;
    // 3. Canonical Ordering
    if mint_a > mint_b {
        println!("mint Canonical swapping....");
        std::mem::swap(&mut mint_a, &mut mint_b);
        std::mem::swap(&mut mint_a_id, &mut mint_b_id);
        std::mem::swap(&mut amount_a, &mut amount_b);
        mints_swapped = true;
    }

    // 4. Create Pool
    create_pool(ctx, &mint_a, &mint_b)?;

    // 5. Fund Alice
    let alice = ctx.alice.pubkey();

    let alice_token_a = get_associated_token_address(&alice, &mint_a);

    let alice_token_b = get_associated_token_address(&alice, &mint_b);

    mint_tokens(ctx, &alice, &alice_token_a, mint_a_id, alice_mint_amount)?;

    mint_tokens(ctx, &alice, &alice_token_b, mint_b_id, alice_mint_amount)?;

    // 6. Derive Pool Infra - to add the initial liquidity
    let (pool, _) = find_pool_pda(&program_id, &mint_a, &mint_b);

    let vault_a = get_associated_token_address(&pool, &mint_a);

    let vault_b = get_associated_token_address(&pool, &mint_b);

    let (lp_mint, _) = find_lp_mint_pda(&program_id, &pool);

    let (locked_lp_token, _) = find_locked_lp_token_pda(&program_id, &pool);

    let (protocol_treasury, _) = find_protocol_treasury_pda(&ctx.program_id);
    let treasury_a = get_associated_token_address(&protocol_treasury, &mint_a);
    let treasury_b = get_associated_token_address(&protocol_treasury, &mint_b);

    // 7. Add Initial Liquidity
    add_initial_liquidity(ctx, &mint_a, &mint_b, amount_a, amount_b)?;

    let pool_struct = InitializedPoolStruct {
        pool,
        mint_a,
        mint_b,
        mint_a_id,
        mint_b_id,

        vault_a,
        vault_b,
        lp_mint,
        locked_lp_token,

        treasury_a,
        treasury_b,
    };

    Ok((pool_struct, mints_swapped))
}

pub fn add_liquidity_as_user(
    ctx: &mut TestContext,
    pool_struct: &InitializedPoolStruct,
    user: &Pubkey,
    max_a: u64,
    max_b: u64,
) -> Result<PoolReservesSnapshot, FailedTransactionMetadata> {
    add_liquidity(
        ctx,
        user,
        &pool_struct.mint_a,
        &pool_struct.mint_b,
        max_a,
        max_b,
    )?;

    let pool_snapshot = pool_reserves_snapshot(ctx, &pool_struct.pool);

    Ok(pool_snapshot)
}

pub fn swap_as_user(
    ctx: &mut TestContext,
    pool_struct: &InitializedPoolStruct,
    user: &Pubkey,
    amount_in: u64,
    min_amount_out: u64,
    a_to_b: bool,
) -> Result<PoolReservesSnapshot, FailedTransactionMetadata> {
    swap(
        ctx,
        user,
        &pool_struct.mint_a,
        &pool_struct.mint_b,
        amount_in,
        min_amount_out,
        a_to_b,
    )?;

    let pool_snapshot = pool_reserves_snapshot(ctx, &pool_struct.pool);

    Ok(pool_snapshot)
}

pub fn derive_user_pool_accounts(
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    lp_mint: &Pubkey,
    user: &Pubkey,
) -> UserPoolAccounts {
    let token_a = get_associated_token_address(user, mint_a);

    let token_b = get_associated_token_address(user, mint_b);

    let lp_token = get_associated_token_address(user, lp_mint);

    UserPoolAccounts {
        token_a,
        token_b,
        lp_token,
    }
}

pub fn pool_reserves_snapshot(ctx: &TestContext, pool_address: &Pubkey) -> PoolReservesSnapshot {
    let pool = pool(ctx, pool_address);

    let vault_a = token_account(ctx, &pool.vault_a);
    let vault_b = token_account(ctx, &pool.vault_b);
    let lp_mint = mint(ctx, &pool.lp_mint);

    let (protocol_treasury, _) = find_protocol_treasury_pda(&ctx.program_id);
    let treasury_a = token_account(
        ctx,
        &get_associated_token_address(&protocol_treasury, &pool.mint_a),
    );
    let treasury_b = token_account(
        ctx,
        &get_associated_token_address(&protocol_treasury, &pool.mint_b),
    );

    PoolReservesSnapshot {
        pool: *pool_address,
        reserve_a: vault_a.amount,
        reserve_b: vault_b.amount,
        lp_supply: lp_mint.supply,
        treasury_a_amount: treasury_a.amount,
        treasury_b_amount: treasury_b.amount,
    }
}

pub fn fund_tokens_to_user(
    ctx: &mut TestContext,
    user: &Pubkey,
    mint_id: u64,
    amount: u64,
) -> Pubkey {
    let (mint, _) = find_mint_pda(&ctx.program_id, mint_id);

    let ata = get_associated_token_address(user, &mint);

    mint_tokens(ctx, user, &ata, mint_id, amount).expect("failed to fund user");

    ata
}

pub fn remove_liquidity_as_user(
    ctx: &mut TestContext,
    pool_struct: &InitializedPoolStruct,
    provider: &Pubkey,
    lp_amount: u64,
    min_amount_a: u64,
    min_amount_b: u64,
) -> Result<PoolReservesSnapshot, FailedTransactionMetadata> {
    remove_liquidity(
        ctx,
        provider,
        &pool_struct.mint_a,
        &pool_struct.mint_b,
        lp_amount,
        min_amount_a,
        min_amount_b,
    )?;

    let pool_reserves = pool_reserves_snapshot(ctx, &pool_struct.pool);

    Ok(pool_reserves)
}
