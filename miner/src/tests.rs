#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, RuntimeOrigin, RuntimeEvent, MaxUrlLength, System, Miner};
    use crate::mock::Test;
    use crate::mock::Whitelist;
    use crate::Pallet as MinerPallet;
    use crate::Error;
    use pallet_whitelist::Pallet as WhitelistPallet;
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
    fn register_miner_works() {
        new_test_ext().execute_with(|| {

            System::deposit_event(RuntimeEvent::Miner(crate::Event::MinerRegistered {
                miner: AccountId32::new([1; 32]),
                deposit: 100,
            }));
            
            println!("System events: {:?}", System::events());

            let miner_id = AccountId32::new([1; 32]);
            let deposit = 100;

            // Register the miner
            assert_ok!(Miner::register_miner(RuntimeOrigin::signed(miner_id.clone()), deposit));

            // Check storage
            let stored_deposit = Miner::miners(&miner_id);
            println!("Stored Deposit: {:?}", stored_deposit);
            assert_eq!(stored_deposit, Some(deposit));

            // Check emitted events
            let events = System::events();
            println!("Events: {:?}", events);

            //Assert that the correct event was emitted
            assert!(events.iter().any(|record| matches!(
               &record.event,
               RuntimeEvent::Miner(crate::Event::MinerRegistered { miner, deposit: dep }) if *miner == miner_id && *dep == deposit
            )));
        });
    }

    #[test]
    fn submit_hash_fails_for_non_whitelisted_url() {
        new_test_ext().execute_with(|| {
            let miner_id = AccountId32::new([1; 32]);
            let url = b"http://example.com".to_vec();
            let hash = H256::random();
            assert_noop!(
                Miner::submit_hash(RuntimeOrigin::signed(miner_id), url, hash),
                Error::<Test>::NotWhitelisted
            );

            // Check emitted events
            let events = System::events();
            println!("Events: {:?}", events);
        });
    }

    #[test]
    fn submit_hash_works_for_whitelisted_url() {
        init_logging();

        new_test_ext().execute_with(|| {
            let miner_id = AccountId32::new([1; 32]);
            let url = b"http://example.com".to_vec();
            let hash = H256::random();

            log::info!("Test start: submit_hash_works_for_whitelisted_url");

            // Add URL to whitelist
            assert_ok!(WhitelistPallet::<Test>::add_url(
                RuntimeOrigin::root(),
                url.clone()
            ));
            //System::finalize();
            //System::initialize(&1, &Default::default(), &Default::default()); // Start a new block
            //log::info!("State finalized after add_url");

            let storage_contents: Vec<_> = pallet_whitelist::Whitelist::<Test>::iter().collect();
            log::info!("Storage state after add_url: {:?}", storage_contents);

            // Verify whitelist storage
            assert!(
                WhitelistPallet::<Test>::is_whitelisted(url.clone()).unwrap(),
                "URL should be in the whitelist"
            );

            // Submit hash
            assert_ok!(Miner::submit_hash(
                RuntimeOrigin::signed(miner_id.clone()),
                url.clone(),
                hash
            ));
            log::info!("Hash successfully submitted");
        });
    }
}
