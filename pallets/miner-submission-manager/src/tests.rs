#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, RuntimeOrigin, RuntimeEvent, MaxUrlLength, MinerSubmissionManager, System,};
    use pallet_whitelist::Pallet as WhitelistPallet;
    use crate::mock::Test;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok, BoundedVec};
    use sp_core::H256;
    use sp_runtime::AccountId32;

    fn init_logging() {
        use std::sync::Once;
        static INIT: Once = Once::new();
    
        INIT.call_once(|| {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Info) // Set the logging level
                .is_test(true) // Indicate this is for tests
                .init();
        });
    }

    #[test]
    fn submit_hash_works() {
        new_test_ext().execute_with(|| {
            let miner_id = AccountId32::new([1; 32]);
            let url = b"http://example.com".to_vec();
            let hash = H256::random();

            // Add URL to whitelist
            let bounded_url: BoundedVec<u8, <Test as crate::Config>::MaxUrlLength> =
                url.clone().try_into().unwrap();

            pallet_whitelist::Pallet::<Test>::add_url(RuntimeOrigin::root(), url.clone());

            // Submit hash
            assert_ok!(MinerSubmissionManager::submit_hash(RuntimeOrigin::signed(miner_id.clone()), url.clone(), hash));

            // Ensure submission is stored
            assert_eq!(
                MinerSubmissionManager::submissions(hash),
                Some((miner_id.clone(), bounded_url.clone()))
            );

            // Check emitted events
            let events = System::events();
            println!("Events: {:?}", events);

            // Check events
            let events = System::events();
            assert!(events.iter().any(|record| matches!(
                &record.event,
                RuntimeEvent::MinerSubmissionManager(crate::Event::SubmissionReceived { miner, .. }) if *miner == miner_id
            )));
        });
    }

    #[test]
    fn submit_hash_fails_for_duplicate_submission() {
        init_logging();
        new_test_ext().execute_with(|| {
            let miner_id = AccountId32::new([1; 32]);
            let url = b"http://example.com".to_vec();
            let hash = H256::random();

            // Add URL to whitelist
            let bounded_url: BoundedVec<u8, <Test as crate::Config>::MaxUrlLength> =
                url.clone().try_into().unwrap();
            pallet_whitelist::Pallet::<Test>::add_url(RuntimeOrigin::root(), url.clone());

            // First submission
            assert_ok!(MinerSubmissionManager::submit_hash(RuntimeOrigin::signed(miner_id.clone()), url.clone(), hash));

            // Duplicate submission
            assert_noop!(
                MinerSubmissionManager::submit_hash(RuntimeOrigin::signed(miner_id.clone()), url.clone(), hash),
                crate::Error::<Test>::DuplicateSubmission
            );
        });
    }

    #[test]
    fn submit_hash_fails_for_unwhitelisted_url() {
        init_logging();
        new_test_ext().execute_with(|| {
            let miner_id = AccountId32::new([1; 32]);
            let url = b"http://unlisted.com".to_vec();
            let hash = H256::random();

            // Attempt to submit without whitelisting
            assert_noop!(
                MinerSubmissionManager::submit_hash(RuntimeOrigin::signed(miner_id.clone()), url, hash),
                crate::Error::<Test>::NotWhitelisted
            );
        });
    }
}
