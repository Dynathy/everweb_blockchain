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
    use sp_runtime::traits::{CheckedMul, CheckedDiv, Zero, CheckedSub};
    use pallet_treasury::Pallet as Treasury;


    //type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type BalanceOf<T> = <<T as pallet_treasury::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);


	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_treasury::Config {
        //type Currency: Currency<Self::AccountId>;
        type RootOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        type TreasuryCurrency: Currency<Self::AccountId>;
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
        type VerifierRewardPercentage: Get<u8>;
        #[pallet::constant]
        type DefaultDevAccount: Get<Self::AccountId>;
        #[pallet::constant]
        type TotalReward: Get<BalanceOf<Self>>;
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
            reward_amount: BalanceOf<T>,
            treasury_amount: BalanceOf<T>,
            developer_amount: BalanceOf<T>,
        },
		RewardsDistributed {
            miner: T::AccountId,
            verifiers: Vec<T::AccountId>, // Reflects multiple verifiers
            miner_reward: BalanceOf<T>,
            verifier_reward: BalanceOf<T>,
        },
	}

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
		///Invalid reward split configuration.
		InvalidFeeSplit,
        InvalidRewardSplit,
        NoVerifiersAssigned, // New Error
        FundsUnavailable,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(12_000)]
        pub fn direct_reward_distribution(
            origin: OriginFor<T>,
            miner: T::AccountId,
            verifiers: Vec<T::AccountId>,
            total_reward: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;
    
            log::info!(
                "Starting direct_reward_distribution: Miner: {:?}, Verifiers: {:?}, Total Reward: {:?}",
                miner,
                verifiers,
                total_reward
            );
    
            // Ensure the reward is not zero
            ensure!(total_reward > Zero::zero(), Error::<T>::InvalidRewardSplit);
    
            // Get treasury and developer accounts
            let treasury_account = pallet_treasury::Pallet::<T>::account_id();
            let dev_account = T::DevPalletId::get().into_account_truncating();
    
            // Calculate fees and remaining reward
            let total_fee = Self::calculate_fee(total_reward)?;
            let dev_fee = total_fee;
            let remaining_reward = Self::calculate_remaining_reward(total_reward, total_fee)?;
    
            // Transfer developer fee
            Self::transfer_developer_fee(&treasury_account, &dev_account, dev_fee)?;
    
            // Distribute miner and verifier rewards
            Self::distribute_rewards(
                &treasury_account,
                miner,
                verifiers,
                remaining_reward,
            )?;
    
            Ok(())
        }
    }
    
    
    impl<T: Config> Pallet<T> {
        fn calculate_fee(total_reward: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
            let fee_split = BalanceOf::<T>::from(100u8 - T::FeeSplitTreasury::get());
            let total_fee = total_reward * fee_split / BalanceOf::<T>::from(100u8);
            log::info!("Calculated Total Fee: {:?}", total_fee);
            Ok(total_fee)
        }
    
        fn calculate_remaining_reward(
            total_reward: BalanceOf<T>,
            total_fee: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let remaining_reward = total_reward.checked_sub(&total_fee)
                .ok_or(Error::<T>::InvalidRewardSplit)?;
            log::info!("Calculated Remaining Reward: {:?}", remaining_reward);
            Ok(remaining_reward)
        }
    }
    impl<T: Config> Pallet<T> {
        fn transfer_developer_fee(
            treasury_account: &T::AccountId,
            dev_account: &T::AccountId,
            dev_fee: BalanceOf<T>,
        ) -> DispatchResult {
            pallet_treasury::Pallet::<T>::transfer_funds(
                frame_system::RawOrigin::Root.into(), // Ensure Root origin
                dev_account.clone(),
                dev_fee,
            )?;
            log::info!(
                "Transferred Developer Fee: {:?} from Treasury: {:?} to Developer: {:?}",
                dev_fee,
                treasury_account,
                dev_account
            );
            Ok(())
        }
    }
    impl<T: Config> Pallet<T> {
        fn distribute_rewards(
            treasury_account: &T::AccountId,
            miner: T::AccountId,
            verifiers: Vec<T::AccountId>,
            remaining_reward: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(!verifiers.is_empty(), Error::<T>::NoVerifiersAssigned);
    
            // Calculate miner's reward
            let miner_reward = remaining_reward
                .checked_mul(&BalanceOf::<T>::from(T::MinerRewardPercentage::get()))
                .and_then(|v| v.checked_div(&BalanceOf::<T>::from(100u32)))
                .ok_or(Error::<T>::InvalidRewardSplit)?;
    
            // Calculate total verifier reward
            let total_verifier_reward = remaining_reward.checked_sub(&miner_reward)
                .ok_or(Error::<T>::InvalidRewardSplit)?;
    
            // Split verifier reward equally
            let per_verifier_reward = total_verifier_reward / BalanceOf::<T>::from(verifiers.len() as u32);
            let remainder = total_verifier_reward % BalanceOf::<T>::from(verifiers.len() as u32);
    
            log::info!("Miner Reward: {:?}", miner_reward);
            log::info!("Verifier Reward: Per Verifier: {:?}, Remainder: {:?}", per_verifier_reward, remainder);
    
            // Transfer miner reward
            pallet_treasury::Pallet::<T>::transfer_funds(
                frame_system::RawOrigin::Root.into(), // Ensure Root origin
                miner.clone(),
                miner_reward,
            )?;
            log::info!("Transferred Miner Reward: {:?} to Miner: {:?}", miner_reward, miner);
    
            // Distribute verifier rewards
            for (i, verifier) in verifiers.iter().enumerate() {
                let reward = if i == 0 {
                    per_verifier_reward + remainder // Add remainder to the first verifier
                } else {
                    per_verifier_reward
                };
    
                pallet_treasury::Pallet::<T>::transfer_funds(
                    frame_system::RawOrigin::Root.into(), // Ensure Root origin
                    verifier.clone(),
                    per_verifier_reward,
                )?;
                log::info!("Transferred Verifier Reward: {:?} to Verifier: {:?}", reward, verifier);
            }
    
            Ok(())
        }
    }    
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;