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
    use pallet_whitelist;
    use pallet_validator as validator; // Import validator pallet


    use frame_system::Pallet as System;
    use sp_io::hashing::blake2_128;
    use rand::rngs::SmallRng;
    use rand::prelude::SliceRandom;
    use rand::SeedableRng;
    
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);


	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_whitelist::Config + validator::Config {
		type Currency: ReservableCurrency<Self::AccountId>;
		type SubmissionFee: Get<BalanceOf<Self>>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
    	type PalletId: Get<PalletId>;
		#[pallet::constant]
    	type MaxUrlLength: Get<u32>; // Maximum length for URLs
	}

    /// Tracks submissions for validation.
    #[pallet::storage]
    #[pallet::getter(fn submissions)]
    pub type Submissions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash,
		(T::AccountId, BoundedVec<u8, <T as Config>::MaxUrlLength>),
		OptionQuery,
	>;

    /// Events emitted by the pallet.
    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		///New Submission Added.
		SubmissionReceived { miner: T::AccountId, hash: T::Hash, url: Vec<u8>},
	}

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
        ///Submission already exists.
		DuplicateSubmission,
		///URL not whitelisted.
		NotWhitelisted,
		///URL exceeds maximum length.
		UrlTooLong
    }

    // Helper function to shuffle validators
    fn shuffle_validators<T: Config>(
        validators: Vec<T::AccountId>,
        seed: &[u8],
    ) -> Vec<T::AccountId> {
        // Generate a deterministic RNG seed
        let mut rng_seed = [0u8; 32];
        rng_seed[..16].copy_from_slice(&blake2_128(seed)); // Copy 16 bytes and pad with zeros
    
        // Create a small RNG instance
        let mut rng = SmallRng::from_seed(rng_seed);
    
        // Shuffle the validators
        let mut shuffled_validators = validators.clone();
        shuffled_validators.shuffle(&mut rng);
    
        shuffled_validators
    }

	/// Pallet calls.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Miner submits a hash for validation.
        #[pallet::weight(10_000)]
        pub fn submit_hash(
            origin: OriginFor<T>,
            url: Vec<u8>,
            hash: T::Hash,
        ) -> DispatchResult {
            let miner = ensure_signed(origin)?;

            log::info!("Start Submission manager submit hash");

            // Convert `url` to `BoundedVec`
            let bounded_url: BoundedVec<u8, <T as Config>::MaxUrlLength> =
                url.clone().try_into().map_err(|_| Error::<T>::UrlTooLong)?;

            log::info!("Pass url to external whitelist");
            // Ensure the URL is whitelisted using the external WhitelistPallet
            ensure!(
                pallet_whitelist::Pallet::<T>::is_whitelisted(url.clone())?,
                Error::<T>::NotWhitelisted
            );


            // Ensure submission is unique
            ensure!(
                !Submissions::<T>::contains_key(&hash),
                Error::<T>::DuplicateSubmission
            );

            // Insert submission
            Submissions::<T>::insert(hash, (miner.clone(), bounded_url));

            // Get all validators
            let all_validators: Vec<_> = validator::Pallet::<T>::validators_iter().collect();
            let total_validators = all_validators.len();

            // Ensure enough validators are available
            ensure!(
                total_validators >= 3,
                "Not enough validators available to assign"
            );

            // Extract only the account IDs from all_validators
            let validator_accounts: Vec<T::AccountId> = all_validators
                .into_iter()
                .map(|(account, _)| account) // Extract the account ID
                .collect();

            // Shuffle validators using the submission hash as a seed
            let shuffled_validators = shuffle_validators::<T>(validator_accounts, hash.as_ref());
            let num_to_assign = (shuffled_validators.len()).min(10).max(3); // Choose between 3 and 10

            // Assign to the top `num_to_assign` validators from the shuffled list
            for validator in shuffled_validators.iter().take(num_to_assign) {
                validator::Pallet::<T>::assign_submission(validator.clone(), hash)?;
                log::info!("Assigned submission {:?} to validator {:?}", hash, validator);
            }

            // Emit an event
            Self::deposit_event(Event::SubmissionReceived { miner, hash, url });

            Ok(())
        }
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;