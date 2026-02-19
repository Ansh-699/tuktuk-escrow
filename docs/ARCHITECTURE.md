# Tuktuk Escrow - Architecture

## Overview

Tuktuk Escrow enables trustless peer-to-peer token exchange with optional automated refunds. The maker locks tokens and receives taker's funds on settlement, or gets an automatic refund if settlement doesn't occur.

![Architecture Diagram](./assets/tuktukescorw.png)

## Accounts

| Account | Purpose |
|---------|----------|
| **Escrow Config PDA** | Stores escrow state and settings |
| **Vault** | Holds locked maker tokens |
| **Maker** | Initiator; receives taker funds or refund |
| **Taker** | Counterparty; must send funds to settle |
| **Queue Authority PDA** | Signs Tuktuk scheduler tasks |

## User Stories

### Trustless Token Exchange
As a maker, I need my tokens locked until the taker pays, so we both commit fairly.
- Maker locks tokens in program-controlled vault
- Taker cannot access vault tokens  
- Settlement releases both sides
- No intermediary needed

### Automatic Settlement After Lock Period
As a user, I need settlement to happen automatically after the lock expires.
- Settlement executes after time lock
- Either party can trigger
- Escrow atomically transitions to SETTLED

### Automatic Refund if Settlement Fails
As a maker, if taker doesn't pay, I need my tokens back automatically.
- Refund releases tokens to maker
- Can be triggered by maker or any actor after deadline
- Vault closes and rent is reclaimed

### Scheduled Automatic Refund
As an operator, I need refunds to execute automatically via scheduler.
- Tuktuk task enqueued at initialization
- Scheduler executes refund at deadline
- No manual intervention required
- Maker recovers funds without extra transactions

## Core Instructions

| Instruction | Action |
|------------|--------|
| **Initialize** | Lock maker tokens, create escrow |
| **Settle** | Release vault tokens to taker |
| **Refund** | Return tokens to maker, close vault |
| **Schedule Refund** | Enqueue task with Tuktuk for auto-refund |
| **Crank Refund** | Execute scheduled refund (Tuktuk invoked) |

## Technology Stack

- **Blockchain**: Solana
- **Framework**: Anchor 0.31.1
- **Token Standard**: Token-2022
- **Scheduler**: Tuktuk v0.3.2
