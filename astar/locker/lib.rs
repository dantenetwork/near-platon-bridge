#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod locker {
    use ink_sdk::{
        Ownable,
        CrossChainSQoS,
        MultiDestContracts,
        cross_chain_helper,
    };
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use payload::message_define::{
        IRequestMessage,
        ISQoS,
        ISQoSType,
        IContent,
        IContext,
    };
    use payload::message_protocol::{
        InkAddressData,
        MsgDetail,
        MessagePayload,
    };
    use ink_storage::{
        Mapping,
        traits::SpreadAllocate,
    };

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq, Clone, Copy, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Erc20Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
        NotOwner,
        BalanceNotEnough,
    }

    const MINT_TOKEN_SELECTOR: [u8; 4] = [0xcf, 0xdd, 0x9a, 0xa2];
    const BURN_TOKEN_SELECTOR: [u8; 4] = [0xb1, 0xef, 0xc1, 0x7b];
    
    #[derive(::scale::Encode, ::scale::Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        MethodNotRegisterd,
        Erc20Error(Erc20Error),
        CrossContractCallError,
        TokenContractNotSet,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Locker {
        /// Account id of owner
        owner: Option<AccountId>,
        cross_chain_contract: Option<AccountId>,
        dante_token_contract: Option<AccountId>,
        dest_contract_map: Mapping<(String, String), (String, String)>,
    }

    /// We use `CrossChainBase` here, to be able to use the sdk functionalities.
    impl cross_chain_helper::CrossChainBase for Locker {
        fn get_cross_chain_contract_address(& self) -> AccountId {
            self.cross_chain_contract.unwrap()
        }
    }

    /// We need access control.
    impl Ownable for Locker {
        /// Returns the account id of the current owner
        #[ink(message)]
        fn owner(& self) -> Option<AccountId> {
            self.owner
        }

        /// Renounces ownership of the contract
        #[ink(message)]
        fn renounce_ownership(&mut self) -> Result<(), u8> {
            self.only_owner()?;

            self.owner = None;

            Ok(())
        }

        /// Transfer ownership to a new account id
        #[ink(message)]
        fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<(), u8> {
            self.only_owner()?;

            self.owner = Some(new_owner);

            Ok(())
        }
    }

    /// We use `MultiDestContracts` of SDK here, to be able to send messages to multi chains.
    impl MultiDestContracts for Locker {      
        #[ink(message)]  
        fn get_dest_contract_info(& self, chain_name: String, action: String) -> Option<(String, String)> {
            self.dest_contract_map.get((chain_name, action))
        }

        #[ink(message)]
        fn register_dest_contract(&mut self, chain_name: String, action: String, contract: String, dest_action: String) -> Result<(), u8> {
            self.only_owner()?;

            self.dest_contract_map.insert((chain_name, action), &(contract, dest_action));

            Ok(())
        }
    }

    /// We use `CrossChainSQoS` here, because
    impl CrossChainSQoS for Locker {
        /// Inserts one SQoS item.
        /// If the item exists, it will be replaced.
        #[ink(message)]
        fn insert(&mut self, sqos_item: ISQoS) -> Result<(), u8> {
            self.only_owner()?;
            
            let mut sqos = cross_chain_helper::get_sqos(self);
            for i in 0..sqos.len() {
                if sqos_item.t == sqos[i].t {
                    return Err(1);
                }
            }
            sqos.push(sqos_item);
            cross_chain_helper::set_sqos(self, sqos);
            Ok(())
        }

        /// Removes one SQoS item.
        #[ink(message)]
        fn remove(&mut self, sqos_type: ISQoSType) -> Result<(), u8> {
            self.only_owner()?;

            let mut sqos = cross_chain_helper::get_sqos(self);
            for i in 0..sqos.len() {
                if sqos[i].t == sqos_type {
                    sqos.remove(i);
                    break;
                }
            }
            cross_chain_helper::set_sqos(self, sqos);

            Ok(())
        }

        /// Clear all SQoS items.
        #[ink(message)]
        fn clear(&mut self) -> Result<(), u8> {
            self.only_owner()?;
            
            let sqos = Vec::<ISQoS>::new();
            cross_chain_helper::set_sqos(self, sqos);

            Ok(())
        }

        /// Sets SQoS items
        #[ink(message)]
        fn set(&mut self, sqos: Vec<ISQoS>) -> Result<(), u8> {
            self.only_owner()?;
            
            for i in 0..sqos.len() {
                for j in (i + 1)..sqos.len() {
                    if sqos[i].t == sqos[j].t {
                        return Err(1);
                    }
                }
            }
            cross_chain_helper::set_sqos(self, sqos);
            Ok(())
        }

        /// Returns SQoS items
        #[ink(message)]
        fn get(& self) -> Vec<ISQoS> {
            cross_chain_helper::get_sqos(self)
        }
    }

    impl Locker {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract)
            })
        }

        /// Initializes the contract with the specified chain name.
        fn new_init(&mut self) {
            let caller = Self::env().caller();
            self.owner = Some(caller);
        }

        /// Sets cross-chain contract address
        #[ink(message)]
        pub fn set_cross_chain_contract(&mut self, contract: AccountId) -> Result<(), u8> {
            self.only_owner()?;
            
            self.cross_chain_contract = Some(contract);

            Ok(())
        }

        /// Sets token contract address
        #[ink(message)]
        pub fn set_token_contract(&mut self, contract: AccountId) -> Result<(), u8> {
            self.only_owner()?;
            
            self.dante_token_contract = Some(contract);

            Ok(())
        }

        /// If caller is the owner of the contract
        fn only_owner(& self) -> Result<(), u8> {
            let caller = self.env().caller();
            if self.owner.unwrap() != caller {
                return Err(1);
            }

            Ok(())
        }

        /// Sends token to another chain 
        #[ink(message)]
        pub fn transfer_token(&mut self, chain_name: String, to_address: InkAddressData, num: u128) -> Result<(), Error> {
            if self.dante_token_contract.is_none() {
                return Err(Error::TokenContractNotSet);
            }

            let caller = Self::env().caller();
            // Burn token
            let result: Result<(), Erc20Error> = ink_env::call::build_call::<ink_env::DefaultEnvironment>()
            .call_type(
                ink_env::call::Call::new()
                    .callee(self.dante_token_contract.unwrap())
                    .gas_limit(0)
                    .transferred_value(0))
            .exec_input(
                ink_env::call::ExecutionInput::new(ink_env::call::Selector::new(BURN_TOKEN_SELECTOR))
                .push_arg(caller)
                .push_arg(num)
            )
            .returns::<Result<(), Erc20Error>>()
            .fire()
            .ok()
            .ok_or(Error::CrossContractCallError)?;

            if result.is_err() {
                return Err(Error::Erc20Error(result.unwrap_err()));
            }
            
            let dest = self.get_dest_contract_info(chain_name.clone(), String::try_from("receive_token").unwrap()).ok_or(Error::MethodNotRegisterd)?;
            let contract = dest.0;
            let action = dest.1;

            let mut msg_payload = MessagePayload::new();
            msg_payload.push_item(String::try_from("to").unwrap(), MsgDetail::InkAddress(to_address));
            msg_payload.push_item(String::try_from("num").unwrap(), MsgDetail::InkU128(num));
            let data = msg_payload.to_bytes();

            let mut sqos = Vec::<ISQoS>::new();
            sqos.push(ISQoS::new(ISQoSType::Reveal, None));
            let content = IContent::new(contract, action, data);
            let message = IRequestMessage::new(chain_name, sqos, content);

            cross_chain_helper::cross_chain_send_message(self, message);

            Ok(())
        }

        /// Receives token from another chain 
        #[ink(message, selector = 0x6DDEC0D1)]
        pub fn receive_token(&mut self, payload: MessagePayload) -> String {
            let addr_item = payload.get_item(String::try_from("to").unwrap()).unwrap();
            let addr = addr_item.in_to::<InkAddressData>().unwrap();
            let num_item = payload.get_item(String::try_from("num").unwrap()).unwrap();
            let num = num_item.in_to::<u128>().unwrap();
            let context: IContext = cross_chain_helper::get_context(self).unwrap();
            
            ink_env::debug_println!("Target address is: {:?}", addr);
            ink_env::debug_println!("Token num is: {:?}", num);

            let to: AccountId = AccountId::try_from(addr.ink_address.unwrap()).unwrap();
            ink_env::call::build_call::<ink_env::DefaultEnvironment>()
            .call_type(
                ink_env::call::Call::new()
                    .callee(self.dante_token_contract.unwrap())
                    .gas_limit(0)
                    .transferred_value(0))
            .exec_input(
                ink_env::call::ExecutionInput::new(ink_env::call::Selector::new(MINT_TOKEN_SELECTOR))
                .push_arg(to)
                .push_arg(num)
            )
            .returns::<()>()
            .fire()
            .unwrap();

            String::new()
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;
        use payload::message_define::{
            ISentMessage,
            ISession,
            ISQoS,
            IContent,
        };

        /// We test if the new constructor does its job.
        #[ink::test]
        fn new_works() {
            let locker = Locker::new();
        }

        /// We test if set_cross_chain_contract works.
        #[ink::test]
        fn set_cross_chain_contract_works() {
            let mut locker = Locker::new();
            let contract_id = ink_env::test::callee::<ink_env::DefaultEnvironment>();
            locker.set_cross_chain_contract(contract_id);
        }
    }
}
