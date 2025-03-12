use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, MintTo};

declare_id!("D1xfJ1FqkCeJ1be5xHgZCTRj2nY2v4c3M3XJXkYmEEvP");

#[program]
pub mod smart_otc_auction {
    use super::*;

    // Initialize program state with auction parameters.
    pub fn initialize(
        ctx: Context<Initialize>,
        reward_mint: Pubkey,
        governance: Pubkey,
        min_bid_increment: u64,
        slippage_tolerance: u64, // expressed in basis points (e.g., 100 = 1%)
        high_value_threshold: u64,
        min_stake: u64,
        reward_vesting_period: i64, // in seconds
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.admin = ctx.accounts.admin.key();
        state.governance = governance;
        state.reward_mint = reward_mint;
        state.auction_count = 0;
        state.min_bid_increment = min_bid_increment;
        state.slippage_tolerance = slippage_tolerance;
        state.high_value_threshold = high_value_threshold;
        state.min_stake = min_stake;
        state.reward_vesting_period = reward_vesting_period;
        Ok(())
    }

    // Place an OTC order into the auction pool.
    pub fn place_order(
        ctx: Context<PlaceOrder>,
        order_type: OrderType,
        base_asset: Pubkey,
        quote_asset: Pubkey,
        amount: u64,
        min_price: u64,
        auction_duration: u64,
        buy_now_price: Option<u64>,
    ) -> Result<()> {
        let order = &mut ctx.accounts.order;
        let state = &mut ctx.accounts.state;

        order.trader = ctx.accounts.trader.key();
        order.order_type = order_type;
        order.base_asset = base_asset;
        order.quote_asset = quote_asset;
        order.amount = amount;
        order.min_price = min_price;
        let current_time = Clock::get()?.unix_timestamp;
        order.auction_end_time = current_time + auction_duration as i64;
        order.start_time = current_time;
        order.highest_bid = 0;
        order.winning_bidder = None;
        order.settled = false;
        order.buy_now_price = buy_now_price;

        state.auction_count = state.auction_count.checked_add(1).unwrap();
        order.id = state.auction_count;

        Ok(())
    }

    // Place a bid on an auctioned OTC order with dynamic bidding rules.
    pub fn place_bid(
        ctx: Context<PlaceBid>,
        auction_id: u64,
        bid_amount: u64,
        expected_price: u64, // used for slippage protection
    ) -> Result<()> {
        let order = &mut ctx.accounts.order;
        let state = &ctx.accounts.state;
        let bidder = &ctx.accounts.bidder;
        let stake_info = &ctx.accounts.stake_info;

        // Ensure auction is still active.
        require!(
            Clock::get()?.unix_timestamp < order.auction_end_time,
            AuctionError::AuctionEnded
        );

        // Slippage protection: ensure bid does not exceed expected price by more than the allowed tolerance.
        let max_allowed = expected_price
            .checked_add(expected_price.checked_mul(state.slippage_tolerance).unwrap() / 10000)
            .unwrap();
        require!(
            bid_amount <= max_allowed,
            AuctionError::ExcessiveSlippage
        );

        // Ensure the bid is at least the minimum required increment.
        let min_next_bid = if order.highest_bid == 0 {
            order.min_price
        } else {
            order.highest_bid.checked_add(state.min_bid_increment).unwrap()
        };
        require!(bid_amount >= min_next_bid, AuctionError::BidTooLow);

        // For high-value bids, require the bidder to have staked a minimum amount.
        if bid_amount >= state.high_value_threshold {
            require!(
                stake_info.amount >= state.min_stake,
                AuctionError::InsufficientStake
            );
        }

        // Update the auction with the new highest bid.
        order.highest_bid = bid_amount;
        order.winning_bidder = Some(bidder.key());

        // Optional "Buy Now": if bid meets/exceeds the buy now price, end the auction immediately.
        if let Some(buy_now) = order.buy_now_price {
            if bid_amount >= buy_now {
                order.auction_end_time = Clock::get()?.unix_timestamp;
            }
        }

        Ok(())
    }

    // Settle the auction once it has ended and update reputation stats.
    pub fn settle_auction(ctx: Context<SettleAuction>, auction_id: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        // Validate auction state in a limited scope to release mutable borrow on order.
        {
            let order = &mut ctx.accounts.order;
            require!(
                current_time >= order.auction_end_time,
                AuctionError::AuctionNotEnded
            );
            require!(
                order.winning_bidder.is_some(),
                AuctionError::NoWinningBid
            );
            require!(!order.settled, AuctionError::AlreadySettled);
        }

        // Copy required fields from order.
        let order_amount = ctx.accounts.order.amount;
        let order_start_time = ctx.accounts.order.start_time;
        let order_trader = ctx.accounts.order.trader;
        let order_winning_bidder = ctx.accounts.order.winning_bidder;

        // Execute asset transfer.
        token::transfer(
            ctx.accounts.transfer_ctx(),
            order_amount,
        )?;

        // Mint rewards with vesting. In production, integrate an actual vesting mechanism.
        token::mint_to(
            ctx.accounts.reward_ctx(),
            10_000_000, // Example reward amount in $SOTC tokens
        )?;

        // Calculate auction duration.
        let auction_duration = current_time - order_start_time;

        // Update reputation-based leaderboard and mark order as settled.
        {
            let order = &mut ctx.accounts.order;
            order.settled = true;
        }
        {
            let trader_stats = &mut ctx.accounts.trader_stats;
            trader_stats.trader = order_trader;
            trader_stats.total_volume = trader_stats.total_volume.checked_add(order_amount).unwrap();
            trader_stats.total_response_time = trader_stats.total_response_time.checked_add(auction_duration).unwrap();
        }
        if let Some(winner) = order_winning_bidder {
            let bidder_stats = &mut ctx.accounts.bidder_stats;
            bidder_stats.trader = winner;
            bidder_stats.win_count = bidder_stats.win_count.checked_add(1).unwrap();
            bidder_stats.total_volume = bidder_stats.total_volume.checked_add(order_amount).unwrap();
            bidder_stats.total_response_time = bidder_stats.total_response_time.checked_add(auction_duration).unwrap();
        }

        Ok(())
    }

    // Admin/Governance function to update auction parameters.
    pub fn update_auction_parameters(
        ctx: Context<UpdateAuctionParameters>,
        min_bid_increment: u64,
        slippage_tolerance: u64,
        high_value_threshold: u64,
        min_stake: u64,
        reward_vesting_period: i64,
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(
            ctx.accounts.updater.key() == state.admin || ctx.accounts.updater.key() == state.governance,
            AuctionError::Unauthorized
        );
        state.min_bid_increment = min_bid_increment;
        state.slippage_tolerance = slippage_tolerance;
        state.high_value_threshold = high_value_threshold;
        state.min_stake = min_stake;
        state.reward_vesting_period = reward_vesting_period;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = 8 + State::SIZE)]
    pub state: Account<'info, State>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(mut)]
    pub trader: Signer<'info>,
    #[account(init, payer = trader, space = 8 + Order::SIZE)]
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub state: Account<'info, State>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(mut)]
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub state: Account<'info, State>,
    // The bidder’s stake account for high-value bids.
    #[account(mut)]
    pub stake_info: Account<'info, StakeInfo>,
}

#[derive(Accounts)]
pub struct SettleAuction<'info> {
    #[account(mut)]
    pub order: Account<'info, Order>,
    #[account(mut)]
    pub state: Account<'info, State>,
    // Token program for asset transfers and minting.
    pub token_program: Program<'info, Token>,
    // Accounts for asset transfer.
    #[account(mut)]
    pub source: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    // Reward minting context.
    #[account(mut)]
    pub reward_mint: Account<'info, Mint>,
    #[account(mut)]
    pub reward_destination: Account<'info, TokenAccount>,
    // Reputation stats accounts for the trader (order creator) and bidder (winning bid).
    #[account(mut, constraint = trader_stats.trader == order.trader)]
    pub trader_stats: Account<'info, TraderStats>,
    #[account(mut)]
    pub bidder_stats: Account<'info, TraderStats>,
}

#[derive(Accounts)]
pub struct UpdateAuctionParameters<'info> {
    #[account(mut)]
    pub updater: Signer<'info>,
    #[account(mut)]
    pub state: Account<'info, State>,
}

#[account]
pub struct State {
    pub admin: Pubkey,
    pub governance: Pubkey,
    pub reward_mint: Pubkey,
    pub auction_count: u64,
    // Auction parameters.
    pub min_bid_increment: u64,
    pub slippage_tolerance: u64, // in basis points
    pub high_value_threshold: u64,
    pub min_stake: u64,
    pub reward_vesting_period: i64,
}

impl State {
    // Approximate size calculation (in bytes).
    const SIZE: usize = 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8;
}

#[account]
pub struct Order {
    pub id: u64,
    pub trader: Pubkey,
    pub order_type: OrderType,
    pub base_asset: Pubkey,
    pub quote_asset: Pubkey,
    pub amount: u64,
    pub min_price: u64,
    pub auction_end_time: i64,
    pub highest_bid: u64,
    pub winning_bidder: Option<Pubkey>,
    pub settled: bool,
    pub buy_now_price: Option<u64>,
    pub start_time: i64,
}

impl Order {
    // Approximate size calculation (adjust as needed).
    const SIZE: usize = 8 + 32 + 1 + 32 + 32 + 8 + 8 + 8 + 8 + 33 + 1 + 9 + 8;
}

#[account]
pub struct TraderStats {
    pub trader: Pubkey,
    pub total_volume: u64,
    pub win_count: u64,
    pub total_response_time: i64,
}

#[account]
pub struct StakeInfo {
    pub owner: Pubkey,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum OrderType {
    Buy,
    Sell,
}

#[error_code]
pub enum AuctionError {
    #[msg("The auction has already ended.")]
    AuctionEnded,
    #[msg("Bid amount is too low.")]
    BidTooLow,
    #[msg("Auction has not ended yet.")]
    AuctionNotEnded,
    #[msg("No winning bid found.")]
    NoWinningBid,
    #[msg("Unauthorized action.")]
    Unauthorized,
    #[msg("Excessive slippage detected.")]
    ExcessiveSlippage,
    #[msg("Insufficient stake for high-value bid.")]
    InsufficientStake,
    #[msg("Auction already settled.")]
    AlreadySettled,
}

// Helper implementations for SettleAuction to build CPI contexts.
impl<'info> SettleAuction<'info> {
    pub fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source.to_account_info().clone(),
            to: self.destination.to_account_info().clone(),
            // Here we assume `order` is a PDA authority. Adjust as needed.
            authority: self.order.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }

    pub fn reward_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.reward_mint.to_account_info().clone(),
            to: self.reward_destination.to_account_info().clone(),
            // Here we assume `order` is the authority. Adjust as needed.
            authority: self.order.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}
