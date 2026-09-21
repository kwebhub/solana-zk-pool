use super::*;

#[derive(Accounts)]
pub struct InitPool<'info> {
    #[account(
        init,
        payer = authority,
        space = PoolState::DISCRIMINATOR.len() + PoolState::INIT_SPACE,
        seeds = [POOL_SEED],
        bump
    )]
    pub pool: Account<'info, PoolState>,

    /// CHECK: Vault for storing SOL.
    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler_pool(ctx: Context<InitPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.authority = ctx.accounts.authority.key();
    pool.next_leaf_index = 0;
    pool.total_deposits = 0;
    pool.current_root_index = 0;
    pool.roots = [EMPTY_ROOT; ROOT_HISTORY_SIZE];

    msg!("Pool initialized with admin: {}", pool.authority);
    Ok(())
}
