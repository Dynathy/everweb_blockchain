#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, RuntimeOrigin, RuntimeEvent, MaxUrlLength, MinerSubmissionManager, System,};
    use crate::mock::Test;
    use crate::Whitelist;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok, BoundedVec};
    use sp_core::H256;
    use sp_runtime::AccountId32;

    #[test]
    fn submit_hash_works() {
        new_test_ext().execute_with(|| {
            let miner_id = AccountId32::new([1; 32]);
            let url = b"http://example.com".to_vec();
            let hash = H256::random();

            // Add URL to whitelist
            let bounded_url: BoundedVec<u8, <Test as crate::Config>::MaxUrlLength> =
                url.clone().try_into().unwrap();
            Whitelist::<Test>::insert(&bounded_url, ());

            // Submit hash
            assert_ok!(MinerSubmissionManager::submit_hash(RuntimeOrigin::signed(miner_id.clone()), url.clone(), hash));

            // Ensure submission is stored
            assert_eq!(
                MinerSubmissionManager::submissions(hash),
                Some((miner_id.clone(), bounded_url.clone()))
            );

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
        new_test_ext().execute_with(|| {
            let miner_id = AccountId32::new([1; 32]);
            let url = b"http://example.com".to_vec();
            let hash = H256::random();

            // Add URL to whitelist
            let bounded_url: BoundedVec<u8, <Test as crate::Config>::MaxUrlLength> =
                url.clone().try_into().unwrap();
            Whitelist::<Test>::insert(&bounded_url, ());

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

    #[test]
    fn add_to_whitelist_works() {
        new_test_ext().execute_with(|| {
            let url = b"http://example.com".to_vec();

            // Add URL to whitelist
            let bounded_url: BoundedVec<u8, <Test as crate::Config>::MaxUrlLength> =
                url.clone().try_into().unwrap();
            Whitelist::<Test>::insert(&bounded_url, ());

            // Ensure URL is whitelisted
            assert!(Whitelist::<Test>::contains_key(&bounded_url));
        });
    }

    #[test]
    fn remove_from_whitelist_works() {
        new_test_ext().execute_with(|| {
            let url = b"http://example.com".to_vec();

            // Add URL to whitelist
            let bounded_url: BoundedVec<u8, <Test as crate::Config>::MaxUrlLength> =
                url.clone().try_into().unwrap();
            Whitelist::<Test>::insert(&bounded_url, ());

            // Remove URL from whitelist
            Whitelist::<Test>::remove(&bounded_url);

            // Ensure URL is no longer whitelisted
            assert!(!Whitelist::<Test>::contains_key(&bounded_url));
        });
    }
}
