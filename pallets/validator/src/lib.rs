#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::pallet::*;

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

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency: ReservableCurrency<Self::AccountId>;
		type SubmissionFee: Get<BalanceOf<Self>>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
    	type PalletId: Get<PalletId>;
		#[pallet::constant]
    	type MaxUrlLength: Get<u32>; // Maximum length for URLs
	}

    /// Tracks registered validators.
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

	/// Tracks assigned submissions to validators.
	#[pallet::storage]
	#[pallet::getter(fn assigned_submissions)]
	pub type AssignedSubmissions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, // Validator Account ID
		BoundedVec<T::Hash, T::MaxUrlLength>, // List of assigned submission hashes with max length
		ValueQuery
	>;

    /// Events emitted by the pallet.
    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ValidatorRegistered { validator: T::AccountId, deposit: BalanceOf<T> },
		/// Submission assigned to a validator.
		SubmissionAssigned { validator: T::AccountId, hash: T::Hash },
		/// Validation completed by a validator.
		ValidationCompleted { validator: T::AccountId, hash: T::Hash, valid: bool },
	}

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
        ValidatorAlreadyRegistered,
        ValidatorNotRegistered,
        NotWhitelisted,
		SubmissionNotAssigned,
        InsufficientFunds,
		UrlTooLong, // Too Long error

    }

	impl<T: Config> Pallet<T> {
		/// Returns the account ID for the pallet
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

    /// Pallet calls.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new validator with a deposit.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_validator(
			origin: OriginFor<T>, 
			deposit: BalanceOf<T>
		) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::ValidatorAlreadyRegistered);

            T::Currency::reserve(&who, deposit)?;
            Validators::<T>::insert(&who, deposit);
			log::info!("About to deposit event for miner registration");
            Self::deposit_event(Event::ValidatorRegistered { validator: who.clone(), deposit });
			log::info!("Event deposited");
            Ok(())
        }

		/// Validators validate a submission.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn validate_submission(
			origin: OriginFor<T>,
			hash: T::Hash,
			is_valid: bool, // True if the submission matches the scrape
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			log::info!("Validator {:?} is attempting to validate submission {:?}", who, hash);

			// Ensure validator is registered
			ensure!(Validators::<T>::contains_key(&who), Error::<T>::ValidatorNotRegistered);

			log::info!("Validator {:?} is registered.", who);

			// Ensure the submission was assigned to this validator
			let assignments = AssignedSubmissions::<T>::get(&who);
			ensure!(assignments.contains(&hash), Error::<T>::SubmissionNotAssigned);

			log::info!("Submission {:?} was assigned to validator {:?}.", hash, who);
			
			// Emit event and notify treasury manager
			Self::deposit_event(Event::ValidationCompleted { validator: who.clone(), hash, valid: is_valid });

			log::info!(
				"Validation completed for submission {:?} by validator {:?}. Valid: {}",
				hash,
				who,
				is_valid
			);

			Ok(())
		}
	}
	impl<T: Config> Pallet<T> {
		pub fn assign_submission(validator: T::AccountId, hash: T::Hash) -> DispatchResult {

			log::info!(
				"Assigning submission {:?} to validator {:?}.",
				hash,
				validator
			);

			let mut assignments = AssignedSubmissions::<T>::get(&validator);

			log::info!(
				"Current assignments for validator {:?}: {:?}",
				validator,
				assignments
			);

			assignments.try_push(hash).map_err(|_| Error::<T>::UrlTooLong)?;
			log::info!(
				"Submission {:?} added to assignments for validator {:?}.",
				hash,
				validator
			);

			AssignedSubmissions::<T>::insert(&validator, assignments);
			log::info!(
				"Assignments for validator {:?} updated in storage.",
				validator
			);
			
			// Emit event for submission assignment
			Self::deposit_event(Event::SubmissionAssigned { validator: validator.clone(), hash });
			log::info!(
				"Event emitted: Submission {:?} assigned to validator {:?}.",
				hash,
				validator
			);

			Ok(())
		}
	
		pub fn validators_iter() -> impl Iterator<Item = (T::AccountId, BalanceOf<T>)> {
			log::info!("Iterating over validators.");
			Validators::<T>::iter()
		}
	}
}
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;