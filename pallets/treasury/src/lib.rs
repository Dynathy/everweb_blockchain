#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, Get},
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::AccountIdConversion;

// Type alias for balance using the Currency trait
type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Currency used for transfers and balances
        type Currency: Currency<Self::AccountId>;

        /// Event type used in the runtime
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Pallet ID for treasury
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn treasury_balance)]
    pub type TreasuryBalance<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Funds have been deposited into the treasury
        FundsDeposited { who: T::AccountId, amount: BalanceOf<T> },
        /// Funds have been transferred out of the treasury
        FundsTransferred { recipient: T::AccountId, amount: BalanceOf<T> },
        /// Rewards have been distributed
        RewardsDistributed {
            miner: T::AccountId,
            validator: T::AccountId,
            miner_reward: BalanceOf<T>,
            validator_reward: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Not enough funds in the treasury to complete the action
        InsufficientFunds,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)] // Static weight for deposit_funds
        pub fn deposit_funds(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let treasury_account = Self::account_id();
            let treasury_balance_before = T::Currency::free_balance(&treasury_account);
            log::info!(
                "Before deposit: Treasury account = {:?}, Balance = {:?}",
                treasury_account,
                treasury_balance_before
            );

            // Transfer funds to treasury
            T::Currency::transfer(
                &who,
                &Self::account_id(),
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            TreasuryBalance::<T>::mutate(|balance| *balance += amount);

            let treasury_balance_after = T::Currency::free_balance(&treasury_account);
            log::info!(
                "After deposit: Treasury account = {:?}, New Balance = {:?}",
                treasury_account,
                treasury_balance_after
            );

            Self::deposit_event(Event::FundsDeposited { who, amount });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(8_000)] // Static weight for transfer_funds
        pub fn transfer_funds(
            origin: OriginFor<T>,
            recipient: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            // Ensure the call is made with Root origin
            ensure_root(origin)?;
        
            // Synchronize TreasuryBalance storage with on-chain balance
            TreasuryBalance::<T>::put(T::Currency::free_balance(&Self::account_id()));
        
            // Retrieve treasury account and balances
            let treasury_account = Self::account_id();
            let treasury_balance_before = T::Currency::free_balance(&treasury_account);
            let recipient_balance_before = T::Currency::free_balance(&recipient);
        
            // Log balances before transfer
            log::info!(
                "Before transfer: Treasury account = {:?}, Balance = {:?}, Recipient = {:?}, Balance = {:?}",
                treasury_account,
                treasury_balance_before,
                recipient,
                recipient_balance_before
            );
        
            // Ensure the treasury has enough funds
            ensure!(
                treasury_balance_before >= amount,
                Error::<T>::InsufficientFunds
            );
        
            // Attempt the transfer
            match T::Currency::transfer(
                &treasury_account,
                &recipient,
                amount,
                ExistenceRequirement::KeepAlive,
            ) {
                Ok(_) => {
                    // Update TreasuryBalance storage
                    TreasuryBalance::<T>::mutate(|balance| *balance -= amount);
        
                    // Log balances after transfer
                    let treasury_balance_after = T::Currency::free_balance(&treasury_account);
                    let recipient_balance_after = T::Currency::free_balance(&recipient);
        
                    log::info!(
                        "After transfer: Treasury account = {:?}, New Balance = {:?}, Recipient = {:?}, New Balance = {:?}",
                        treasury_account,
                        treasury_balance_after,
                        recipient,
                        recipient_balance_after
                    );
        
                    // Emit success event
                    Self::deposit_event(Event::FundsTransferred { recipient, amount });
                }
                Err(err) => {
                    // Log transfer failure
                    log::error!(
                        "Transfer failed: Treasury account = {:?}, Recipient = {:?}, Amount = {:?}, Error = {:?}",
                        treasury_account,
                        recipient,
                        amount,
                        err
                    );
                    return Err(err);
                }
            }
        
            Ok(())
        }        

        #[pallet::call_index(2)]
        #[pallet::weight(12_000)] // Static weight for distribute_rewards
        pub fn distribute_rewards(
            origin: OriginFor<T>,
            miner: T::AccountId,
            validator: T::AccountId,
            miner_reward: BalanceOf<T>,
            validator_reward: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let total_reward = miner_reward + validator_reward;

            let treasury_account = Self::account_id();
            let treasury_balance_before = T::Currency::free_balance(&treasury_account);

            log::info!(
                "Before rewards distribution: Treasury account = {:?}, Balance = {:?}, Total Reward = {:?}",
                treasury_account,
                treasury_balance_before,
                total_reward
            );
            ensure!(
                TreasuryBalance::<T>::get() >= total_reward,
                Error::<T>::InsufficientFunds
            );

            // Transfer rewards
            T::Currency::transfer(
                &Self::account_id(),
                &miner,
                miner_reward,
                ExistenceRequirement::AllowDeath,
            )?;
            T::Currency::transfer(
                &Self::account_id(),
                &validator,
                validator_reward,
                ExistenceRequirement::AllowDeath,
            )?;

            TreasuryBalance::<T>::mutate(|balance| *balance -= total_reward);

            let treasury_balance_after = T::Currency::free_balance(&treasury_account);
            log::info!(
                "After rewards distribution: Treasury account = {:?}, New Balance = {:?}, Miner = {:?}, Validator = {:?}",
                treasury_account,
                treasury_balance_after,
                miner,
                validator
            );

            Self::deposit_event(Event::RewardsDistributed {
                miner,
                validator,
                miner_reward,
                validator_reward,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get the account ID for the treasury
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

