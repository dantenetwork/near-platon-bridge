use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
// use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};
use near_sdk::{ext_contract, serde_json, Gas};
use protocol_sdk::{Content, Context, OmniChain, Payload, Value};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Locker {
    omni_chain: OmniChain,
    token_contract_id: AccountId,
}

#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_mint(&mut self, receiver_id: AccountId, amount: U128);
    fn ft_burn(&mut self, account_id: AccountId, amount: U128);
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    DestinationContract,
    PermittedContract,
}

#[derive(Clone, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(tag = "type", crate = "near_sdk::serde")]
pub struct SendMessage {
    receiver: String,
    to_chain: String,
}

#[near_bindgen]
impl Locker {
    #[init]
    pub fn new(
        owner_id: AccountId,
        omni_chain_contract_id: AccountId,
        token_contract_id: AccountId,
    ) -> Self {
        Self {
            omni_chain: OmniChain::new(
                owner_id,
                StorageKey::DestinationContract,
                StorageKey::PermittedContract,
                omni_chain_contract_id,
            ),
            token_contract_id,
        }
    }

    // #[warn(unused_variables)]
    pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128 {
        assert_eq!(
            env::predecessor_account_id(),
            self.token_contract_id,
            "only receive token create by {}",
            self.token_contract_id
        );
        let action_name = "transfer_token".to_string();
        let message: SendMessage = serde_json::from_str(&msg).unwrap();
        let dst_contract = self
            .omni_chain
            .destination_contract
            .get(&message.to_chain)
            .expect("to chain not register");
        let contract = dst_contract
            .get(&action_name)
            .expect("contract not register");
        let mut payload = Payload::new();
        let address: String = message.receiver.as_str().chars().skip(2).collect();
        let address_bytes = hex::decode(address);
        if address_bytes.is_ok() {
            payload.push_item("to".to_string(), Value::VecUint8(address_bytes.unwrap()));
        } else {
            payload.push_item("to".to_string(), Value::String(message.receiver))
        }
        payload.push_item("num".to_string(), Value::Uint128(amount));
        let content = Content {
            contract: contract.contract_address.clone(),
            action: contract.action_name.clone(),
            data: payload,
        };
        self.omni_chain.call_cross(message.to_chain, content);
        ext_ft::ft_burn(
            env::current_account_id(),
            amount,
            self.token_contract_id.clone(),
            1,
            Gas(5_000_000_000_000),
        );
        U128(0)
    }

    pub fn transfer(&mut self, payload: Payload, context: Context) {
        assert_eq!(
            env::predecessor_account_id(),
            self.omni_chain.omni_chain_contract_id,
            "Processs by cross chain contract"
        );
        self.omni_chain.assert_register_permitted_contract(
            &context.from_chain,
            &context.sender,
            &context.action,
        );

        let receiver_id_item = payload.get_item("to".to_string()).unwrap();
        let receiver_id: AccountId = receiver_id_item
            .get_value::<String>()
            .unwrap()
            .parse()
            .unwrap();
        let amount_item = payload.get_item("num".to_string()).unwrap();
        let amount = amount_item.get_value::<U128>().unwrap();
        ext_ft::ft_mint(
            receiver_id,
            amount,
            self.token_contract_id.clone(),
            1,
            Gas(5_000_000_000_000),
        );
    }
}

protocol_sdk::impl_omni_chain_register!(Locker, omni_chain);
