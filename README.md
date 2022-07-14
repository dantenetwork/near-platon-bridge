# near-platon-bridge
This repository implements the token bridge between Near and PlatON on their testnets.

## Prerequisite
- npm: >= 6.14.16
- node: >= v14.19.1

## Install
### PlatON
```
cd platon
npm install
```

## Usage

### Prepare Accounts
You must have an account on each network. You can refer to the following information to create accounts and get native tokens.

**PlatON**

PlatON is evm-compatible, so you can use metamask to create an account, the rpc address is https://devnetopenapi2.platon.network/rpc, and the port is 2203181. Then you can get tokens from the [faucet](https://faucet.platon.network/faucet/).

[Here](https://devdocs.platon.network/docs/zh-CN/Join_Dev_Network) is more information about PlatON Development.

Put the private key to the `.secret` file.
```
cd platon
echo <PRIVATE_KEY> > .secret
```

Replace `<PRIVATE_KEY>` with your private key.

**Near**

Go to the [testnet](https://wallet.testnet.near.org/create) to create an account.

instll NEAR CLI (command line interface)

```sh
npm install -g near-cli
```
### Get DAT Test Tokens

set token contract and locker contract:

```sh
export TOKEN="16edad39fa9e1a6ae33ce3f35b7c301dbf974e2594b3d26f16c00c3105853751"
export LOCKER="ae997e9b674480cb9bc88765242c3cbf71267044b51a2b8f99f20028aa441d89"
```

get test token:

```sh
near call $TOKEN ft_faucet --accountId RECEIVER_ID --depositYocto 1
```

### Transfer DAT from Near to PlatON

Call the token contract `ft_transfer_call` method, and `receiver_id` is locker contract; `amount` is the amount you want to transfer; `msg` must be a string of json object, with two keys, `receiver` is the address you want receive, and `to_chain` is the chain you receive token.

example:

```sh
near call $TOKEN ft_transfer_call '{"receiver_id": "'${LOCKER}'", "amount":"1000000000000000000", "msg": "{\"receiver\":\"0x3aE841B899Ae4652784EA734cc61F524c36325d1\",\"to_chain\":\"PLATON\"}"}' --accountId $token --depositYocto 1 --gas 65000000000000
```

**Query Results**

- Near: https://wallet.testnet.near.org/
- PlatON: Import DAT token in the Metamask, the contract address is *0x5639983D7B9d0e5a0f3998a2A34a496718d93936*.

![image](https://user-images.githubusercontent.com/83757490/178889548-791eb7e4-0407-483a-a99c-66e288356613.png)


### Transfer DAT from PlatON to Near

```
cd platon
node register/registerLocker.js -s PLATON,NEARTEST,<NEAR_ACCOUNT>,<TOKEN_NUM>
```

`<NEAR_ACCOUNT>` is the address on Near, which will receive the DAT tokens.  
`<TOKEN_NUM>` is the number of DAT that will be transferred to Near, **the decimal is 18**, so you must set 1000000000000000000 if you want to transfer 1 DAT.

Then query results.
