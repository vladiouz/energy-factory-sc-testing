#![allow(non_snake_case)]

mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};


const GATEWAY: &str = sdk::blockchain::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";


#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "lockTokens" => interact.lock_tokens_endpoint().await,
        "unlockTokens" => interact.unlock_tokens_endpoint().await,
        "extendLockPeriod" => interact.extend_lock_period().await,
        "issueLockedToken" => interact.issue_locked_token().await,
        "getLockedTokenId" => interact.locked_token().await,
        "getBaseAssetTokenId" => interact.base_asset_token_id().await,
        "getLegacyLockedTokenId" => interact.legacy_locked_token_id().await,
        "getEnergyEntryForUser" => interact.get_updated_energy_entry_for_user().await,
        "getEnergyAmountForUser" => interact.get_energy_amount_for_user().await,
        "addLockOptions" => interact.add_lock_options().await,
        "getLockOptions" => interact.get_lock_options_view().await,
        "unlockEarly" => interact.unlock_early().await,
        "reduceLockPeriod" => interact.reduce_lock_period().await,
        "getPenaltyAmount" => interact.calculate_penalty_amount().await,
        "setTokenUnstakeAddress" => interact.set_token_unstake_address().await,
        "revertUnstake" => interact.revert_unstake().await,
        "getTokenUnstakeScAddress" => interact.token_unstake_sc_address().await,
        "setEnergyForOldTokens" => interact.set_energy_for_old_tokens().await,
        "updateEnergyAfterOldTokenUnlock" => interact.update_energy_after_old_token_unlock().await,
        "migrateOldTokens" => interact.migrate_old_tokens().await,
        "pause" => interact.pause_endpoint().await,
        "unpause" => interact.unpause_endpoint().await,
        "isPaused" => interact.paused_status().await,
        "setTransferRoleLockedToken" => interact.set_transfer_role().await,
        "setBurnRoleLockedToken" => interact.set_burn_role().await,
        "mergeTokens" => interact.merge_tokens_endpoint().await,
        "lockVirtual" => interact.lock_virtual().await,
        "addSCAddressToWhitelist" => interact.add_sc_address_to_whitelist().await,
        "removeSCAddressFromWhitelist" => interact.remove_sc_address_from_whitelist().await,
        "isSCAddressWhitelisted" => interact.is_sc_address_whitelisted().await,
        "addToTokenTransferWhitelist" => interact.add_to_token_transfer_whitelist().await,
        "removeFromTokenTransferWhitelist" => interact.remove_from_token_transfer_whitelist().await,
        "setUserEnergyAfterLockedTokenTransfer" => interact.set_user_energy_after_locked_token_transfer().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}


#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    contract_address: Option<Bech32Address>
}

impl State {
        // Deserializes state from file
        pub fn load_state() -> Self {
            if Path::new(STATE_FILE).exists() {
                let mut file = std::fs::File::open(STATE_FILE).unwrap();
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                toml::from_str(&content).unwrap()
            } else {
                Self::default()
            }
        }
    
        /// Sets the contract address
        pub fn set_address(&mut self, address: Bech32Address) {
            self.contract_address = Some(address);
        }
    
        /// Returns the contract address
        pub fn current_address(&self) -> &Bech32Address {
            self.contract_address
                .as_ref()
                .expect("no known contract, deploy first")
        }
    }
    
    impl Drop for State {
        // Serializes state to file
        fn drop(&mut self) {
            let mut file = std::fs::File::create(STATE_FILE).unwrap();
            file.write_all(toml::to_string(self).unwrap().as_bytes())
                .unwrap();
        }
    }

struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    state: State
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::alice());
        
        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/energy-factory.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state()
        }
    }

    async fn deploy(&mut self) {
        let base_asset_token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let legacy_token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let old_locked_asset_factory_address = bech32::decode("");
        let min_migrated_token_locked_period = 0u64;
        let lock_options = MultiValueVec::from(vec![MultiValue2::from((0u64, 0u64))]);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .typed(proxy::SimpleLockEnergyProxy)
            .init(base_asset_token_id, legacy_token_id, old_locked_asset_factory_address, min_migrated_token_locked_period, lock_options)
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_address(Bech32Address::from_bech32_string(new_address_bech32.clone()));

        println!("new address: {new_address_bech32}");
    }

    async fn lock_tokens_endpoint(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let lock_epochs = 0u64;
        let opt_destination = OptionalValue::Some(bech32::decode(""));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .lock_tokens_endpoint(lock_epochs, opt_destination)
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn unlock_tokens_endpoint(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .unlock_tokens_endpoint()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn extend_lock_period(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let lock_epochs = 0u64;
        let user = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .extend_lock_period(lock_epochs, user)
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn issue_locked_token(&mut self) {
        let egld_amount = BigUint::<StaticApi>::from(0u128);

        let token_display_name = ManagedBuffer::new_from_bytes(&b""[..]);
        let token_ticker = ManagedBuffer::new_from_bytes(&b""[..]);
        let num_decimals = 0u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .issue_locked_token(token_display_name, token_ticker, num_decimals)
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn locked_token(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .locked_token()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn base_asset_token_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .base_asset_token_id()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn legacy_locked_token_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .legacy_locked_token_id()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_updated_energy_entry_for_user(&mut self) {
        let user = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .get_updated_energy_entry_for_user(user)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn get_energy_amount_for_user(&mut self) {
        let user = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .get_energy_amount_for_user(user)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn add_lock_options(&mut self) {
        let new_lock_options = MultiValueVec::from(vec![MultiValue2::from((0u64, 0u64))]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .add_lock_options(new_lock_options)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn get_lock_options_view(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .get_lock_options_view()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn unlock_early(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .unlock_early()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn reduce_lock_period(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let new_lock_period = 0u64;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .reduce_lock_period(new_lock_period)
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn calculate_penalty_amount(&mut self) {
        let token_amount = BigUint::<StaticApi>::from(0u128);
        let prev_lock_epochs = 0u64;
        let new_lock_epochs = 0u64;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .calculate_penalty_amount(token_amount, prev_lock_epochs, new_lock_epochs)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn set_token_unstake_address(&mut self) {
        let sc_address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .set_token_unstake_address(sc_address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn revert_unstake(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let user = bech32::decode("");
        let new_energy = PlaceholderInput;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .revert_unstake(user, new_energy)
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn token_unstake_sc_address(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .token_unstake_sc_address()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn set_energy_for_old_tokens(&mut self) {
        let users_energy = PlaceholderInput;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .set_energy_for_old_tokens(users_energy)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn update_energy_after_old_token_unlock(&mut self) {
        let original_caller = bech32::decode("");
        let initial_epoch_amount_pairs = PlaceholderInput;
        let final_epoch_amount_pairs = PlaceholderInput;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .update_energy_after_old_token_unlock(original_caller, initial_epoch_amount_pairs, final_epoch_amount_pairs)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn migrate_old_tokens(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .migrate_old_tokens()
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn pause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .pause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn unpause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .unpause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn paused_status(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn set_transfer_role(&mut self) {
        let opt_address = OptionalValue::Some(bech32::decode(""));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .set_transfer_role(opt_address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_burn_role(&mut self) {
        let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .set_burn_role(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn merge_tokens_endpoint(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let opt_original_caller = OptionalValue::Some(bech32::decode(""));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .merge_tokens_endpoint(opt_original_caller)
            .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn lock_virtual(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let amount = BigUint::<StaticApi>::from(0u128);
        let lock_epochs = 0u64;
        let dest_address = bech32::decode("");
        let energy_address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .lock_virtual(token_id, amount, lock_epochs, dest_address, energy_address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_sc_address_to_whitelist(&mut self) {
        let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .add_sc_address_to_whitelist(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_sc_address_from_whitelist(&mut self) {
        let address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .remove_sc_address_from_whitelist(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn is_sc_address_whitelisted(&mut self) {
        let address = bech32::decode("");

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .is_sc_address_whitelisted(address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn add_to_token_transfer_whitelist(&mut self) {
        let sc_addresses = MultiValueVec::from(vec![bech32::decode("")]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .add_to_token_transfer_whitelist(sc_addresses)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_from_token_transfer_whitelist(&mut self) {
        let sc_addresses = MultiValueVec::from(vec![bech32::decode("")]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .remove_from_token_transfer_whitelist(sc_addresses)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_user_energy_after_locked_token_transfer(&mut self) {
        let user = bech32::decode("");
        let energy = PlaceholderInput;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::SimpleLockEnergyProxy)
            .set_user_energy_after_locked_token_transfer(user, energy)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

}
