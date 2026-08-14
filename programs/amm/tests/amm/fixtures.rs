use anchor_spl::associated_token::get_associated_token_address;
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;

use crate::{
    amm::{
        accounts::{mint, pool, token_account},
        add_initial_liquidity::add_initial_liquidity,
        add_liquidity::add_liquidity,
        create_pool::create_pool,
        initialize_mint::initialize_mint,
        initialize_protocol::initialize_protocol,
        mint_tokens::mint_tokens,
        pdas::{find_locked_lp_token_pda, find_lp_mint_pda, find_mint_pda, find_pool_pda},
        structs::{InitializedPoolStruct, PoolReservesSnapshot, UserPoolAccounts},
    },
    common::context::TestContext,
};

pub fn setup_initialized_pool(
    ctx: &mut TestContext,
    alice_mint_amount: u64,
    initial_amount_a: u64,
    initial_amount_b: u64,
) -> Result<InitializedPoolStruct, FailedTransactionMetadata> {
    let program_id = ctx.program_id;

    // 1. Initilize Protocol
    initialize_protocol(ctx)?;

    // 2. Create Test Mints
    let mut mint_a_id = 1_u64;
    let mut mint_b_id = 2_u64;

    initialize_mint(ctx, mint_a_id, 9)?;
    initialize_mint(ctx, mint_b_id, 9)?;

    let (mut mint_a, _) = find_mint_pda(&program_id, mint_a_id);
    let (mut mint_b, _) = find_mint_pda(&program_id, mint_b_id);

    // 3. Canonical Ordering
    if mint_a > mint_b {
        std::mem::swap(&mut mint_a, &mut mint_b);
        std::mem::swap(&mut mint_a_id, &mut mint_b_id);
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

    // 7. Add Initial Liquidity
    add_initial_liquidity(ctx, &mint_a, &mint_b, initial_amount_a, initial_amount_b)?;

    Ok(InitializedPoolStruct {
        pool,
        mint_a,
        mint_b,
        mint_a_id,
        mint_b_id,

        vault_a,
        vault_b,
        lp_mint,
        locked_lp_token,
    })
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

    PoolReservesSnapshot {
        pool: *pool_address,
        reserve_a: vault_a.amount,
        reserve_b: vault_b.amount,
        lp_supply: lp_mint.supply,
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
