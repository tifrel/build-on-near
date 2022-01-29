#!/usr/bin/env bash

cargo test || exit 1
cargo emit || exit 1

wasm='target/wasm32-unknown-unknown/release/near_buy_me_a_coffee.wasm'

near deploy --accountId coffee.tifrel.testnet --wasmFile "$wasm" \
  --initFunction initialize \
  --initArgs '{"owner": "tifrel.testnet"}'
# Starting deployment. Account id: coffee.tifrel.testnet, node: https://rpc.testnet.near.org, helper: https://helper.testnet.near.org, file: target/wasm32-unknown-unknown/release/near_buy_me_a_coffee.wasm
# Transaction Id 25g5CJxyMurDgsSSCuWndkhnuzGc2CPp322LC9hWCiUB
# To see the transaction in the transaction explorer, please open this url in your browser
# https://explorer.testnet.near.org/transactions/25g5CJxyMurDgsSSCuWndkhnuzGc2CPp322LC9hWCiUB
# Done deploying and initializing coffee.tifrel.testnet

near create-account someone.tifrel.testnet \
  --masterAccount tifrel.testnet \
  --initialBalance 5

near create-account sometwo.tifrel.testnet \
  --masterAccount tifrel.testnet \
  --initialBalance 5

near call coffee.tifrel.testnet buy_coffee '{}' \
  --accountId someone.tifrel.testnet \
  --deposit 1
# Scheduling a call: coffee.tifrel.testnet.buy_coffee({}) with attached 1 NEAR
# Doing account.functionCall()
# Transaction Id 7bdK7tRpQUgMGBiYXkXixM4EPpsPqghRBm9kHyqdLJKp
# To see the transaction in the transaction explorer, please open this url in your browser
# https://explorer.testnet.near.org/transactions/7bdK7tRpQUgMGBiYXkXixM4EPpsPqghRBm9kHyqdLJKp
near view coffee.tifrel.testnet coffee_near_from '{"account": "someone.tifrel.testnet"}' --accountId tifrel.testnet
# View call: coffee.tifrel.testnet.coffee_near_from({"account": "someone.tifrel.testnet"})
# 1e+24
near view coffee.tifrel.testnet top_coffee_buyer --accountId tifrel.testnet
# View call: coffee.tifrel.testnet.top_coffee_buyer()
# [ 'someone.tifrel.testnet', 1e+24 ]

near call coffee.tifrel.testnet buy_coffee '{}' \
  --accountId sometwo.tifrel.testnet \
  --deposit 2
# Scheduling a call: coffee.tifrel.testnet.buy_coffee({}) with attached 2 NEAR
# Doing account.functionCall()
# Transaction Id no99p6t2osKCGu1ZeFz7xc5J3TWAnfFNdhvKqbVnVVV
# To see the transaction in the transaction explorer, please open this url in your browser
# https://explorer.testnet.near.org/transactions/no99p6t2osKCGu1ZeFz7xc5J3TWAnfFNdhvKqbVnVVV
# ''
near view coffee.tifrel.testnet coffee_near_from '{"account": "sometwo.tifrel.testnet"}' --accountId tifrel.testnet
# View call: coffee.tifrel.testnet.coffee_near_from({"account": "sometwo.tifrel.testnet"})
# 2e+24
near view coffee.tifrel.testnet top_coffee_buyer --accountId tifrel.testnet
# View call: coffee.tifrel.testnet.top_coffee_buyer()
# [ 'sometwo.tifrel.testnet', 2e+24 ]
