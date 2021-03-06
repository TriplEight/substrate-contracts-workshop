#![cfg_attr(not(any(test, feature = "test-env")), no_std)]

use parity_codec::{
    Decode,
    Encode,
};
use ink_core::{
    env::{
        self,
        AccountId,
        Balance,
    },
    memory::format,
    storage,
};
use ink_lang::contract;

/// Events deposited by the ERC20 token contract.
#[derive(Encode, Decode)]
enum Event {
    Transfer {
        from: Option<AccountId>,
        to: Option<AccountId>,
        value: Balance,
    },
    // ACTION: Create an `Approval` event with:
    //         * owner: AccountId
    //         * spender: AccountId
    //         * value: Balance
}

/// Deposits an ERC20 token event.
fn deposit_event(event: Event) {
    env::deposit_raw_event(&event.encode()[..])
}

contract! {
    /// The storage items for a typical ERC20 token implementation.
    struct Erc20 {
        /// The total supply.
        total_supply: storage::Value<Balance>,
        /// The balance of each user.
        balances: storage::HashMap<AccountId, Balance>,
        /// Balances that are spendable by non-owners: (owner, spender) -> allowed
        // ACTION: Create a new `allowances` HashMap which maps
        //         a tuple `(AccountId, AccountId)` to `Balance`
    }

    impl Deploy for Erc20 {
        fn deploy(&mut self, init_value: Balance) {
            self.total_supply.set(init_value);
            self.balances.insert(env.caller(), init_value);
            deposit_event(Event::Transfer { 
                from: None,
                to: Some(env.caller()),
                value: init_value
            });
        }
    }

    impl Erc20 {
        /// Returns the total number of tokens in existence.
        pub(external) fn total_supply(&self) -> Balance {
            let total_supply = *self.total_supply;
            env.println(&format!("Erc20::total_supply = {:?}", total_supply));
            total_supply
        }

        /// Returns the balance of the given AccountId.
        pub(external) fn balance_of(&self, owner: AccountId) -> Balance {
            let balance = self.balance_of_or_zero(&owner);
            env.println(&format!("Erc20::balance_of(owner = {:?}) = {:?}", owner, balance));
            balance
        }

        /// Returns the amount of tokens that an owner allowed to a spender.
        pub(external) fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            // ACTION: Create a getter for the `allowances` HashMap
            //   HINT: Take a look at the getters above if you forget the details
            // ACTION: Return the `allowance` at the end
        }

        /// Transfers token from the sender to the `to` AccountId.
        pub(external) fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
            self.transfer_impl(env.caller(), to, value)
        }

        /// Approve the passed AccountId to spend the specified amount of tokens
        /// on the behalf of the message's sender.
        pub(external) fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
            // ACTION: Get the `env.caller()` and store it as the `owner`
            // ACTION: Insert the new allowance into the `allowances` HashMap
            //   HINT: The key tuple is `(owner, spender)`
            // ACTION: Deposit the `Approval` event you created using these values
            // ACTION: Return true if everything was successful
        }

        /// Transfer tokens from one AccountId to another.
        pub(external) fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            // ACTION: Get the allowance for `(from, env.caller())` using `allowance_or_zero`
            // ACTION: `if` the `allowance` is less than the `value`, exit early and return `false`
            // ACTION: `insert` the new allowance into the map for `(from, env.caller())`
            // ACTION: Finally, call the `transfer_impl` for `from` and `to`
        }
    }

    impl Erc20 {
        /// Returns the balance of the AccountId or 0 if there is no balance.
        fn balance_of_or_zero(&self, of: &AccountId) -> Balance {
            let balance = self.balances.get(of).unwrap_or(&0);
            *balance
        }

        /// Returns the allowance or 0 of there is no allowance.
        fn allowance_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            // ACTION: Get the allowance between `(owner, spender)` and `unwrap_or` return 0
            // ACTION: Return the allowance
        }

        /// Transfers token from a specified AccountId to another AccountId.
        fn transfer_impl(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            let balance_from = self.balance_of_or_zero(&from);
            let balance_to = self.balance_of_or_zero(&to);
            if balance_from < value {
                return false
            }
            self.balances.insert(from, balance_from - value);
            self.balances.insert(to, balance_to + value);
            deposit_event(Event::Transfer { 
                from: Some(from),
                to: Some(to),
                value: value
            });
            true
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn deployment_works() {
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        env::test::set_caller(alice);

        // Deploy the contract with some `init_value`
        let erc20 = Erc20::deploy_mock(1234);
        // Check that the `total_supply` is `init_value`
        assert_eq!(erc20.total_supply(), 1234);
        // Check that `balance_of` Alice is `init_value`
        assert_eq!(erc20.balance_of(alice), 1234);
    }

    #[test]
    fn transfer_works() {
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let bob = AccountId::try_from([0x1; 32]).unwrap();

        env::test::set_caller(alice);
        // Deploy the contract with some `init_value`
        let mut erc20 = Erc20::deploy_mock(1234);
        // Alice does not have enough funds for this
        assert_eq!(erc20.transfer(bob, 4321), false);
        // Alice can do this though
        assert_eq!(erc20.transfer(bob, 234), true);
        // Check Alice and Bob have the expected balance
        assert_eq!(erc20.balance_of(alice), 1000);
        assert_eq!(erc20.balance_of(bob), 234);
    }

    #[test]
    fn allowance_works() {
        let alice = AccountId::try_from([0x0; 32]).unwrap();
        let bob = AccountId::try_from([0x1; 32]).unwrap();
        let charlie = AccountId::try_from([0x2; 32]).unwrap();

        env::test::set_caller(alice);
        // Deploy the contract with some `init_value`
        let mut erc20 = Erc20::deploy_mock(1234);
        // Bob does not have an allowance from Alice's balance
        assert_eq!(erc20.allowance(alice, bob), 0);
        // Thus, Bob cannot transfer out of Alice's account
        env::test::set_caller(bob);
        assert_eq!(erc20.transfer_from(alice, bob, 1), false);
        // Alice can approve bob for some of her funds
        env::test::set_caller(alice);
        assert_eq!(erc20.approve(bob, 20), true);
        // And the allowance reflects that correctly
        assert_eq!(erc20.allowance(alice, bob), 20);

        // Charlie cannot send on behalf of Bob
        env::test::set_caller(charlie);
        assert_eq!(erc20.transfer_from(alice, bob, 10), false);
        // Bob cannot transfer more than he is allowed
        env::test::set_caller(bob);
        assert_eq!(erc20.transfer_from(alice, charlie, 25), false);
        // A smaller amount should work though
        assert_eq!(erc20.transfer_from(alice, charlie, 10), true);
        // Check that the allowance is updated
        assert_eq!(erc20.allowance(alice, bob), 10);
        // and the balance transferred to the right person
        assert_eq!(erc20.balance_of(charlie), 10);
    }
}
