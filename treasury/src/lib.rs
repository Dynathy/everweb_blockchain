#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::{
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, Get},
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::AccountIdConversion;

use log::debug;

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

            // Transfer funds to treasury
            T::Currency::transfer(
                &who,
                &Self::account_id(),
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            TreasuryBalance::<T>::mutate(|balance| *balance += amount);

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
            ensure_root(origin)?;

            ensure!(
                TreasuryBalance::<T>::get() >= amount,
                Error::<T>::InsufficientFunds
            );

            // Transfer funds out of treasury
            T::Currency::transfer(
                &Self::account_id(),
                &recipient,
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            TreasuryBalance::<T>::mutate(|balance| *balance -= amount);

            Self::deposit_event(Event::FundsTransferred { recipient, amount });

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

            debug!(
                "Attempting to distribute rewards: miner_reward = {:?}, validator_reward = {:?}, total_reward = {:?}",
                miner_reward, validator_reward, total_reward
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