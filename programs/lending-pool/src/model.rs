use anchor_lang::prelude::*;
use anchor_lang::error_code;
use anchor_lang::Event;

use anchor_spl::token::{Mint, TokenAccount, Token};
use anchor_spl::associated_token::AssociatedToken;

#[account]
#[derive(Default, Debug)]
pub struct LendingPool {
    pub mint: Pubkey,              // 抵押品代币的 Mint 地址（USDC）
    pub decimals: u8,              // 代币精度
    pub total_supply: u64,         // 总供应量（存款）
    pub total_borrowed: u64,       // 总借款
    pub liquidity_index: u128,     // 流动性指数（复利计算）
    pub borrow_index: u128,        // 借款指数（复利计算）
    pub reserve_factor: u8,        // 储备金率（如 10%）
    pub collateral_factor: u8,     // 抵押率（如 75%）
    pub last_update_time: i64,     // 最后更新时间戳
    pub base_rate: u64,            // 基础利率（APR）
}


#[derive(Accounts)]
pub struct InitializePool<'info> {
    // ----------------------------
    // 资金池账户
    // ----------------------------
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<LendingPool>(),
        seeds = [b"lending_pool"],
        bump
    )]
    pub pool: Account<'info, LendingPool>,

    // ----------------------------
    // 代币 Mint 账户
    // ----------------------------
    pub mint: Account<'info, Mint>,

    // ----------------------------
    // 权限账户
    // ----------------------------
    #[account(mut)]
    pub authority: Signer<'info>,

    // ----------------------------
    // 系统相关账户
    // ----------------------------
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // ----------------------------
    // 用户相关账户
    // ----------------------------

    /// CHECK: demo 看一下这个是不是可以去掉
    #[account(mut)]
    pub user: Signer<'info>, // 用户签名账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>, // 用户的代币账户

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserPosition>(),
        seeds = [b"user_position", user.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>, // 用户仓位账户

    // ----------------------------
    // 资金池相关账户
    // ----------------------------
    #[account(mut)]
    pub pool: Account<'info, LendingPool>, // 资金池账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Account<'info, TokenAccount>, // 资金池的代币账户

    // ----------------------------
    // 系统相关账户
    // ----------------------------
    pub token_program: Program<'info, Token>, // SPL Token 程序
    pub associated_token_program: Program<'info, AssociatedToken>, // 关联代币程序
    pub system_program: Program<'info, System>, // 系统程序
    pub rent: Sysvar<'info, Rent>, // 租金系统变量
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    // ----------------------------
    // 用户相关账户
    // ----------------------------

    /// CHECK: demo 看一下这个是不是可以去掉
    #[account(mut)]
    pub user: Signer<'info>, // 用户签名账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>, // 用户的代币账户

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserPosition>(),
        seeds = [b"user_position", user.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>, // 用户仓位账户

    // ----------------------------
    // 资金池相关账户
    // ----------------------------
    #[account(mut)]
    pub pool: Account<'info, LendingPool>, // 资金池账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Account<'info, TokenAccount>, // 资金池的代币账户

    // ----------------------------
    // 系统相关账户
    // ----------------------------
    pub token_program: Program<'info, Token>, // SPL Token 程序
    pub associated_token_program: Program<'info, AssociatedToken>, // 关联代币程序
    pub system_program: Program<'info, System>, // 系统程序
    pub rent: Sysvar<'info, Rent>, // 租金系统变量
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    // ----------------------------
    // 用户相关账户
    // ----------------------------

    /// CHECK: demo 看一下这个是不是可以去掉
    #[account(mut)]
    pub user: Signer<'info>, // 用户签名账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>, // 用户的代币账户

    #[account(
        mut,
        seeds = [b"user_position", user.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>, // 用户仓位账户

    // ----------------------------
    // 资金池相关账户
    // ----------------------------
    #[account(mut)]
    pub pool: Account<'info, LendingPool>, // 资金池账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Account<'info, TokenAccount>, // 资金池的代币账户

    // ----------------------------
    // 系统相关账户
    // ----------------------------
    pub token_program: Program<'info, Token>, // SPL Token 程序
    pub system_program: Program<'info, System>, // 系统程序
}

#[derive(Accounts)]
pub struct Repay<'info> {
    // ----------------------------
    // 用户相关账户
    // ----------------------------

    /// CHECK: demo 看一下这个是不是可以去掉
    #[account(mut)]
    pub user: Signer<'info>, // 用户签名账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>, // 用户的代币账户

    #[account(
        mut,
        seeds = [b"user_position", user.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>, // 用户仓位账户

    // ----------------------------
    // 资金池相关账户
    // ----------------------------
    #[account(mut)]
    pub pool: Account<'info, LendingPool>, // 资金池账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Account<'info, TokenAccount>, // 资金池的代币账户

    // ----------------------------
    // 系统相关账户
    // ----------------------------
    pub token_program: Program<'info, Token>, // SPL Token 程序
    pub system_program: Program<'info, System>, // 系统程序
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    // ----------------------------
    // 清算人相关账户
    // ----------------------------
    #[account(mut)]
    pub liquidator: Signer<'info>, // 清算人签名账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = liquidator
    )]
    pub liquidator_token_account: Account<'info, TokenAccount>, // 清算人的代币账户

    // ----------------------------
    // 用户相关账户
    // ----------------------------

    /// CHECK: 被清算的用户
    #[account(mut)]
    pub user: AccountInfo<'info>, 

    #[account(
        mut,
        seeds = [b"user_position", user.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_position: Account<'info, UserPosition>, // 用户仓位账户

    // ----------------------------
    // 资金池相关账户
    // ----------------------------
    #[account(mut)]
    pub pool: Account<'info, LendingPool>, // 资金池账户

    #[account(
        mut,
        associated_token::mint = pool.mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Account<'info, TokenAccount>, // 资金池的代币账户

    // ----------------------------
    // 系统相关账户
    // ----------------------------
    pub token_program: Program<'info, Token>, // SPL Token 程序
    pub system_program: Program<'info, System>, // 系统程序
}

// ----------------------------
// 用户仓位账户（记录每个用户的抵押和借款）
// ----------------------------
#[account]
#[derive(Default, Debug)]
pub struct UserPosition {

    pub user: Pubkey,              // 用户地址
    pub pool: Pubkey,              // 关联的 LendingPool
    pub deposited_amount: u64,     // 抵押品数量
    pub borrowed_amount: u64,      // 借款数量
    pub collateral_enabled: bool,  // 是否启用抵押
    pub last_update_time: i64,     // 用户仓位最后更新时间
}


// ----------------------------
// 事件记录账户（记录关键操作）
// ----------------------------
#[account]
pub struct LendingEvent {
    pub event_type: u8,            // 0=存款, 1=取款, 2=借款, 3=还款, 4=清算
    pub amount: u64,

    pub user: Pubkey,
    pub timestamp: i64,
}

impl Event for LendingEvent {
    fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(1 + 8 + 32 + 8);
        data.push(self.event_type);
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(self.user.as_ref());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data
    }
}
    

#[error_code]
pub enum LendingError {
    // ----------------------------
    // 数学计算错误
    // ----------------------------
    #[msg("Math overflow")]
    MathOverflow,

    // ----------------------------
    // 借贷相关错误
    // ----------------------------
    #[msg("Borrow limit exceeded")]
    BorrowLimitExceeded,

    #[msg("Insufficient collateral")]
    InsufficientCollateral,

    #[msg("Not liquidatable")]
    NotLiquidatable,

    // ----------------------------
    // 权限相关错误
    // ----------------------------
    #[msg("Unauthorized")]
    Unauthorized,

    // ----------------------------
    // 账户相关错误
    // ----------------------------
    #[msg("Account not initialized")]
    AccountNotInitialized,

    #[msg("Account data too small")]
    AccountDataTooSmall,

    // ----------------------------
    // 其他错误
    // ----------------------------
    #[msg("Invalid oracle price")]
    InvalidOraclePrice,

    #[msg("Invalid timestamp")]
    InvalidTimestamp,
}