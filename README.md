# smart_otc_auction
# Smart OTC Auction Token ($SOTC)

## 📌 Overview

**Smart OTC Auction Token ($SOTC)** is an on-chain **Over-the-Counter (OTC) trading system** built on **Solana**. It provides a **real-time, auction-based mechanism** for traders to place large buy/sell orders while liquidity providers bid to execute them at the best price.

This program is developed using **Anchor** and **Solana Playground** and was exported to **VSCODE**. Itrewards market makers with **$SOTC tokens** for participation.

---

## 🔹 How It Works

1. **Traders place OTC orders** into auction pools.
2. **Liquidity providers bid** for order execution by offering price improvements.
3. **The best bid wins**, and the transaction settles **automatically on-chain**.
4. **Market makers earn $SOTC tokens** for participating in OTC auctions.

---

## 🚀 Features

✅ **Dynamic Auction-Based Price Discovery**  
✅ **Encourages competition** among liquidity providers to improve execution prices  
✅ **High-Frequency Execution Ranking** (rewards based on response time)  
✅ **Leaderboard System** ranks traders & market makers based on performance  
✅ **Multi-Asset Support** (SOL, USDC, USDT, etc.)  
✅ **Flash Loan Prevention & Anti-Sybil Mechanisms**  
✅ **Upgradable Governance System**  

---


## 🏗️ Smart Contract Structure

### 1️⃣ **Auction Management**
- `initialize` → Initializes the auction system.
- `place_order` → Places a buy/sell order into an auction pool.
- `place_bid` → Liquidity providers bid to execute the order.
- `settle_auction` → Determines the winning bid and executes the trade.

### 2️⃣ **Dynamic Bidding Rules**
- **Slippage protection** prevents excessive bid deviations.
- **Minimum bid increments** prevent frontrunning.
- **Buy Now price option** enables instant settlement.

### 3️⃣ **Multi-Asset Support**
- Supports **multiple tokens** (SOL, USDC, USDT, etc.).
- Cross-token bidding support (future integration with **oracles** like **Pyth**).

### 4️⃣ **Reputation-Based Leaderboard**
- Tracks **total trading volume, win rate, and response time**.
- **Top-ranked traders and market makers** receive bonus rewards.

### 5️⃣ **Flash Loan Prevention & Anti-Sybil**
- **Staking Requirement**: Large trades require minimum staking.
- **Time-Based Vesting**: $SOTC rewards are **locked** for a set period.

### 6️⃣ **Upgradable & Governance-Controlled**
- Admin/governance account can **update auction parameters**.
- Future integration with **DAO governance** for decentralization.

---

