#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

    pub mod treasury_manager {
        use sp_std::vec::Vec;
        use frame_support::dispatch::DispatchResult;

        pub fn distribute_rewards<T>(
            _miner: T::AccountId,
            _validator_submissions: Vec<(T::AccountId, bool)>,
        ) -> DispatchResult
        where
            T: frame_system::Config,
        {
            // Stub: Add logging or events here as needed for testing.
            Ok(())
        }
    }


    use codec::FullCodec;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::{Currency, Get},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::{fmt::Debug, vec::Vec};

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: Currency<Self::AccountId>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxValidatorSubmissions: Get<u32>;

        #[pallet::constant]
        type ValidationTimeout: Get<BlockNumberFor<Self>>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::storage]
    #[pallet::getter(fn validator_submissions)]
    pub type ValidatorSubmissions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        BoundedVec<(T::AccountId, bool), T::MaxValidatorSubmissions>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validation_deadline)]
    pub type ValidationDeadline<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn processed_submissions)]
    pub type ProcessedSubmissions<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn miner_for_hash)]
    pub type MinerForHash<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        T::AccountId,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SubmissionValidated {
            miner: T::AccountId,
            hash: T::Hash,
            valid: bool,
        },
        SubmissionExpired {
            hash: T::Hash,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        SubmissionAlreadyProcessed,
        ValidationExpired,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(n: BlockNumberFor<T>) {
            for (hash, deadline) in ValidationDeadline::<T>::iter() {
                log::info!("on_finalize called at block {}: checking hash {:?}", n, hash);

                // Skip submissions that have already been processed
                if ProcessedSubmissions::<T>::contains_key(&hash) {
                    log::info!("Skipping already processed submission: {:?}", hash);
                    continue;
                }

                // Retrieve submissions for the given hash
                let submissions = ValidatorSubmissions::<T>::get(&hash);

                // If the submission has sufficient entries and is before the deadline, process it
                if submissions.len() > 0 && n <= deadline {
                    log::info!("Processing valid submissions for hash: {:?}", hash);

                    if let Some(miner) = MinerForHash::<T>::get(&hash) {
                        Self::process_submissions(miner.clone(), hash, submissions.to_vec());
                        ProcessedSubmissions::<T>::insert(&hash, ());
                        log::info!("Submission marked as processed: {:?}", hash);
                    } else {
                        log::warn!("No miner found for hash: {:?}", hash);
                    }
                    continue; // Skip further checks for this submission
                }

                // Handle expiration if the deadline has passed and the submission is unprocessed
                if n > deadline {
                    log::info!("Submission expired: {:?}", hash);
                    Self::handle_expired_submission(hash);
                    ProcessedSubmissions::<T>::insert(&hash, ());
                    log::info!("Submission marked as expired: {:?}", hash);
                } else {
                    log::info!("Submission still valid and awaiting further validations: {:?}", hash);
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn submit_validation(
            origin: OriginFor<T>,
            miner: T::AccountId,
            hash: T::Hash,
            is_valid: bool,
        ) -> DispatchResult {
            let validator = ensure_signed(origin)?;
            
            ///Ensure that when a validation is submitted the miner is stored
            if MinerForHash::<T>::get(&hash).is_none() {
                MinerForHash::<T>::insert(&hash, &miner);
            }

            ValidatorSubmissions::<T>::try_mutate(&hash, |submissions| -> Result<(), DispatchError> {
                if let Err(_) = submissions.try_push((validator.clone(), is_valid)) {
                    // Handle case where the BoundedVec is full
                    return Err(DispatchError::Other("ValidatorSubmissions is full"));
                }
                Ok(())
            })?;

            ensure!(
                !ProcessedSubmissions::<T>::contains_key(&hash),
                Error::<T>::SubmissionAlreadyProcessed
            );
        
            let current_block = <frame_system::Pallet<T>>::block_number();
            if let Some(deadline) = ValidationDeadline::<T>::get(&hash) {
                ensure!(current_block <= deadline, Error::<T>::ValidationExpired);
            } else {
                ValidationDeadline::<T>::insert(&hash, current_block + T::ValidationTimeout::get());
            }
        
            // Fetch the updated submissions for further processing
            let submissions = ValidatorSubmissions::<T>::get(&hash);
            if submissions.len() >= T::MaxValidatorSubmissions::get() as usize {
                Self::process_submissions(miner, hash, submissions.to_vec());
                ProcessedSubmissions::<T>::insert(&hash, ());
            }
        
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn process_submissions(
            miner: T::AccountId,
            hash: T::Hash,
            submissions: Vec<(T::AccountId, bool)>,
        ) {
            log::info!("ValidatorSubmissions for hash {:?}: {:?}", hash, submissions);
            log::info!("Processing submissions for hash: {:?}", hash);
            let valid_count = submissions.iter().filter(|(_, valid)| *valid).count();
            let total_count = submissions.len();
            log::info!("Submissions length: {}", submissions.len());
            // Calculate 2/3 threshold
            if valid_count as f32 >= (2.0 / 3.0) * total_count as f32 {
                log::info!(
                    "2/3 threshold met: {} out of {} validators signaled valid",
                    valid_count,
                    total_count
                );
                Self::handle_valid_submission(hash, miner, submissions);
            } else {
                log::info!(
                    "2/3 threshold not met: {} out of {} validators signaled invalid",
                    total_count - valid_count,
                    total_count
                );
                Self::handle_invalid_submission(hash, miner);
            }
    
            // Mark submission as processed
            log::info!("Marking submission as processed: {:?}", hash);
            ProcessedSubmissions::<T>::insert(hash, ());
        }
    

        fn handle_valid_submission(hash: T::Hash, miner: T::AccountId, submissions: Vec<(T::AccountId, bool)>) {
            treasury_manager::distribute_rewards::<T>(miner.clone(), submissions.clone())
                .expect("Stub logic");
            Self::deposit_event(Event::SubmissionValidated {
                miner,
                hash,
                valid: true,
            });
        }

        fn handle_invalid_submission(hash: T::Hash, miner: T::AccountId) {
            Self::deposit_event(Event::SubmissionValidated {
                miner,
                hash,
                valid: false,
            });
        }

        fn handle_expired_submission(hash: T::Hash) {
            log::info!("Inserting hash into ProcessedSubmissions: {:?}", hash);
            ValidationDeadline::<T>::remove(&hash);
            MinerForHash::<T>::remove(&hash); // Clean up miner entry
            ProcessedSubmissions::<T>::insert(&hash, ());
            log::info!("Hash inserted into ProcessedSubmissions: {:?}", hash);
            Self::deposit_event(Event::SubmissionExpired { hash });
        }
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;