<h1>Spelltroum Tournament Betting Contract</h1>

This repository contains an Anchor-based Solana smart contract that implements the betting logic for the Spelltroum game. The contract is deployed on DevNet and is called from the Spelltroum game client when players place bets or settle matches.

<h3>Overview</h3>

Players deposit SOL into their on-chain balance, play a 2–3 minute match in the game, and then, after a victory, a trusted backend service invokes the contract to distribute rewards. Once they have accumulated enough SOL, players can withdraw their balance.

All bets and rewards are denominated in SOL.

<h3>Instruction Methods</h3>

1. deposit

Invoked directly from the game client using the Solana Mobile Stack, this instruction:
	<li>Validates that the deposit amount is greater than zero.</li>
	<li>Initializes or updates the player’s on-chain balance account.</li>
	<li>Emits a Deposited event with the user’s public key and the deposited amount.</li>

2. withdraw

Still under development, this instruction will provide a transparent way for players to withdraw their SOL. Further design and testing are required before production use.

3. settle_match

Implements the core reward distribution logic:
	<li>Calculates the total prize pool based on the entry fee and number of players.</li>
	<li>Splits the pool evenly among winners.</li>
	<li>Deducts entry fees from each player’s balance.</li>
	<li>Credits winnings to each winner’s balance.</li>
	<li>Records the match ID in a circular history to prevent double settlements.</li>

Currently open to the public for local testing. In the future, this instruction will be restricted to calls from a trusted backend authority when distributing rewards post-match. Additional work and testing are pending.

<h3>Deployment</h3>
	<li>Deployed to: DevNet</li>
	<li>Program ID: Ayt8CLKegBbfHHGkFmMqutNepCAfWzNdo3zPxdqLnQws</li>

⸻

Note: This contract is under active development. Use at your own risk and always test thoroughly on DevNet before mainnet deployment.
