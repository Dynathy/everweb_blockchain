#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
		BoundedVec,
		PalletId,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
	use sp_runtime::traits::AccountIdConversion;
    use sp_runtime::traits::{CheckedMul, CheckedDiv, Zero};

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency: ReservableCurrency<Self::AccountId>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
    	type TreasuryPalletId: Get<PalletId>;
		#[pallet::constant]
    	type DevPalletId: Get<PalletId>;
		#[pallet::constant]
    	type FeeSplitTreasury: Get<u8>; //remaining precentage goes to developer
        #[pallet::constant]
        type MinerRewardPercentage: Get<u8>;
        #[pallet::constant]
        type ValidatorRewardPercentage: Get<u8>;
        #[pallet::constant]
        type DefaultDevAccount: Get<Self::AccountId>;
	}

	/// Tracks the total fees collected by the Treasury Manager.
    #[pallet::storage]
    #[pallet::getter(fn total_fees_collected)]
    pub type TotalFeesCollected<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Events emitted by the pallet.
    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		FeesAllocated {
            treasury_amount: BalanceOf<T>,
            developer_amount: BalanceOf<T>,
        },
		RewardsDistributed {
            miner: T::AccountId,
            validators: Vec<T::AccountId>, // Reflects multiple validators
            miner_reward: BalanceOf<T>,
            validator_reward: BalanceOf<T>,
        },
	}

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
		///Invalid reward split configuration.
		InvalidFeeSplit,
        InvalidRewardSplit,
        NoValidatorsAssigned, // New Error
    }

	#[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Direct the Treasury to distribute rewards.
        #[pallet::call_index(0)]
        #[pallet::weight(12_000)]
        pub fn direct_reward_distribution(
            origin: OriginFor<T>,
            miner: T::AccountId,
            validators: Vec<T::AccountId>,
            total_reward: BalanceOf<T>,
        ) -> DispatchResult {
            // Ensure the caller has root privileges.
            ensure_root(origin)?;

            // **NEW**: Retrieve treasury and developer accounts.
            let treasury_account = T::TreasuryPalletId::get().into_account_truncating();
           // let dev_account = T::DevPalletId::get()
           //     .try_into_account()
           //     .unwrap_or_else(|| T::DefaultDevAccount::get());

           let dev_account = T::DefaultDevAccount::get();

            // Ensure Not Zero
            ensure!(total_reward > Zero::zero(), Error::<T>::InvalidRewardSplit);
            // **NEW**: Calculate and transfer the developer fee.
            let dev_fee = total_reward 
                * (BalanceOf::<T>::from(100u8) - BalanceOf::<T>::from(T::FeeSplitTreasury::get())) 
                / BalanceOf::<T>::from(100u8);

            log::info!(
                "Developer Fee Transfer: From Treasury: {:?} to Developer: {:?} Amount: {:?}",
                treasury_account,
                dev_account,
                dev_fee
            );

            T::Currency::transfer(
                &treasury_account,
                &dev_account,
                dev_fee,
                ExistenceRequirement::AllowDeath,
            )?;

            // Adjust the total reward to exclude the developer fee.
            let remaining_reward = total_reward - dev_fee;
        
            // Validate the reward split configuration.
            ensure!(
                T::MinerRewardPercentage::get() + T::ValidatorRewardPercentage::get() == 100,
                Error::<T>::InvalidRewardSplit
            );
        
            // Ensure validators are assigned.
            ensure!(!validators.is_empty(), Error::<T>::NoValidatorsAssigned);
        
            // Calculate miner's reward.
            let miner_reward = remaining_reward
                .checked_mul(&BalanceOf::<T>::from(T::MinerRewardPercentage::get()))
                .and_then(|v| v.checked_div(&BalanceOf::<T>::from(100u32)))
                .ok_or(Error::<T>::InvalidRewardSplit)?;
        
            // Calculate total validator rewards and split among validators.
            let total_validator_reward = remaining_reward - miner_reward;
            let per_validator_reward = total_validator_reward / BalanceOf::<T>::from(validators.len() as u32);
        
            // **NEW**: Retrieve treasury account to transfer funds from.
            let treasury_account = T::TreasuryPalletId::get().into_account_truncating();
        
            // Transfer miner reward directly using `Currency`.
            T::Currency::transfer(
                &treasury_account,
                &miner,
                miner_reward,
                ExistenceRequirement::AllowDeath,
            )?;
        
            // Transfer rewards to each validator directly using `Currency`.
            for validator in &validators {
                T::Currency::transfer(
                    &treasury_account,
                    validator,
                    per_validator_reward,
                    ExistenceRequirement::AllowDeath,
                )?;
            }
        
            // Emit event to record the reward distribution.
            Self::deposit_event(Event::RewardsDistributed {
                miner,
                validators: validators.clone(), // Clone for event emission
                miner_reward,
                validator_reward: per_validator_reward,
            });

            Self::deposit_event(Event::FeesAllocated {
                treasury_amount: remaining_reward,
                developer_amount: dev_fee,
            });
        
            Ok(())
        }
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;