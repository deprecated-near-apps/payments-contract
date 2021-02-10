use borsh::{ BorshDeserialize, BorshSerialize };
use near_sdk::{
    env, near_bindgen, AccountId, Balance, PublicKey, Promise,
    collections::{ UnorderedMap },
    json_types::{ U128, Base58PublicKey },
};
use serde::Serialize;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]

#[derive(Debug, Serialize, BorshDeserialize, BorshSerialize)]
pub struct Deposit {
    pub memo: String,
    pub amount: U128,
    pub paid: bool,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PaymentContract {
    pub owner_id: AccountId,
    pub deposits: UnorderedMap<AccountId, Vec<Deposit>>,
}

impl Default for PaymentContract {
    fn default() -> Self {
        panic!("Should be initialized before usage")
    }
}

#[near_bindgen]
impl PaymentContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Invalid owner account");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id,
            deposits: UnorderedMap::new(b"deposits".to_vec()),
        }
    }

    #[payable]
    pub fn deposit(&mut self, memo: String) {
        assert!(memo.len() < 64, "memo too long");
        let amount = env::attached_deposit();
        let account_id = env::signer_account_id();
        let mut deposits = self.deposits.get(&account_id).unwrap_or(vec![]);
        deposits.push(Deposit{
            memo,
            amount: amount.into(),
            paid: false,
        });
        self.deposits.insert(&account_id, &deposits);
    }

    pub fn make_payment(&mut self, deposit_index: usize) {
        let account_id = env::signer_account_id();
        let mut deposits = self.deposits.get(&account_id).unwrap_or(vec![]);
        deposits[deposit_index].paid = true;
        self.deposits.insert(&account_id, &deposits);
    }

    pub fn withdraw(&mut self, deposit_index: usize) {
        let account_id = env::signer_account_id();
        let mut deposits = self.deposits.get(&account_id).unwrap_or(vec![]);
        let deposit = deposits.remove(deposit_index);
        assert!(deposit.paid == false, "payment was already confirmed");
        self.deposits.insert(&account_id, &deposits);
        Promise::new(account_id).transfer(deposit.amount.into());
    }

    /// view methods

    pub fn get_deposits(&self, account_id: AccountId) -> Vec<Deposit> {
        self.deposits.get(&account_id).unwrap_or(vec![])
    }
}

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};
    
    fn ntoy(near_amount: u128) -> U128 {
        U128(near_amount * 10u128.pow(24))
    }

    fn get_context() -> VMContext {
        VMContext {
            predecessor_account_id: "alice.testnet".to_string(),
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "bob.testnet".to_string(),
            signer_account_pk: vec![0],
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
            storage_usage: 1000
        }
    }

    #[test]
    fn make_deposit() {
        let mut context = get_context();
        context.attached_deposit = ntoy(1).into();
        testing_env!(context.clone());

        let mut contract = PaymentContract::new(context.current_account_id.clone());
        contract.deposit("take my money".to_string());

        let deposits = contract.get_deposits(context.signer_account_id.clone());


        assert_eq!(deposits.get(0).unwrap().memo, "take my money".to_string());
    }

    #[test]
    fn make_deposit_and_payment() {
        let mut context = get_context();
        context.attached_deposit = ntoy(1).into();
        testing_env!(context.clone());

        let mut contract = PaymentContract::new(context.current_account_id.clone());
        contract.deposit("take my money".to_string());
        contract.make_payment(0);

        let deposits = contract.get_deposits(context.signer_account_id.clone());

        assert_eq!(deposits[0].paid, true);
    }

    #[test]
    fn make_deposit_and_withdrawal() {
        let mut context = get_context();
        context.attached_deposit = ntoy(1).into();
        testing_env!(context.clone());

        let mut contract = PaymentContract::new(context.current_account_id.clone());
        contract.deposit("take my money".to_string());
        contract.withdraw(0);

        let deposits = contract.get_deposits(context.signer_account_id.clone());

        assert_eq!(deposits.len(), 0);
    }

    // #[test]
    // #[should_panic(
    //     expected = r#"Message exists"#
    // )]
    // fn panic_create() {
    //     let mut context = get_context();
    //     context.signer_account_pk = Base58PublicKey::try_from("ed25519:Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap().into();
    //     testing_env!(context.clone());
    //     let mut contract = Messages::new(context.current_account_id.clone());
    //     contract.create("hello world!".to_string(), ntoy(10), "alice.testnet".to_string());
    //     contract.create("hello world!".to_string(), ntoy(10), "alice.testnet".to_string());
    // }
    
    // #[test]
    // #[should_panic(
    //     expected = r#"No message"#
    // )]
    // fn panic_purchase() {
    //     let mut context = get_context();
    //     context.signer_account_pk = Base58PublicKey::try_from("ed25519:Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap().into();
    //     testing_env!(context.clone());
    //     let mut contract = Messages::new(context.current_account_id.clone());
    //     contract.create("hello world!".to_string(), ntoy(10), "alice.testnet".to_string());
    //     context.signer_account_pk = vec![4,5,6];
    //     context.account_balance = ntoy(1000).into();
    //     context.attached_deposit = ntoy(10).into();
    //     testing_env!(context.clone());
    //     contract.purchase(Base58PublicKey::try_from("ed25519:Bg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy").unwrap());
    // }
}