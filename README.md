#Spelltroum Tournament Betting Contract

This repository contains an Anchor-based Solana smart contract that implements the betting logic for the Spelltroum game. The contract is deployed on DevNet and is called from the Spelltroum game client when players place bets or settle matches.

##Overview

Players deposit SOL into their on-chain balance, play a 2–3 minute match in the game, and then, after a victory, a trusted backend service invokes the contract to distribute rewards. Once they have accumulated enough SOL, players can withdraw their balance.

All bets and rewards are denominated in SOL.

##Instruction Methods

1. deposit

Invoked directly from the game client using the Solana Mobile Stack, this instruction:
	•	Validates that the deposit amount is greater than zero.
	•	Initializes or updates the player’s on-chain balance account.
	•	Emits a Deposited event with the user’s public key and the deposited amount.

2. withdraw

Still under development, this instruction will provide a transparent way for players to withdraw their SOL. Further design and testing are required before production use.

3. settle_match

Implements the core reward distribution logic:
	•	Calculates the total prize pool based on the entry fee and number of players.
	•	Splits the pool evenly among winners.
	•	Deducts entry fees from each player’s balance.
	•	Credits winnings to each winner’s balance.
	•	Records the match ID in a circular history to prevent double settlements.

Currently open to the public for local testing. In the future, this instruction will be restricted to calls from a trusted backend authority when distributing rewards post-match. Additional work and testing are pending.

##Deployment
	•	Deployed to: DevNet
	•	Program ID: Ayt8CLKegBbfHHGkFmMqutNepCAfWzNdo3zPxdqLnQws

##Usage
	1.	Setup
	•	Install Anchor and Solana CLI.
	•	Configure your environment to point to DevNet:

solana config set --url https://api.devnet.solana.com

	2.	Build & Deploy

anchor build
anchor deploy

⸻

Note: This contract is under active development. Use at your own risk and always test thoroughly on DevNet before mainnet deployment.