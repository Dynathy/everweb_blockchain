#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, RuntimeOrigin, RuntimeEvent, MaxUrlLength, System, Miner};
    use crate::mock::Test;
    use crate::Whitelist;
    use crate::Pallet as MinerPallet;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok, BoundedVec};
    use sp_core::H256;
    use sp_runtime::AccountId32;



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
        new_test_ext().execute_with(|| {
            let miner_id = AccountId32::new([1; 32]);
            let url = BoundedVec::<u8, MaxUrlLength>::try_from(b"http://example.com".to_vec()).unwrap();
            let hash = H256::random();

            // Insert into Whitelist
            Whitelist::<Test>::insert(&url, ());

            assert_ok!(Miner::submit_hash(RuntimeOrigin::signed(miner_id.clone()), url.clone().into(), hash));
            assert_eq!(Miner::submissions(hash), Some((miner_id, url)));

            // Check emitted events
            let events = System::events();
            println!("Events: {:?}", events);
        });
    }

    #[test]
    fn add_to_whitelist_works() {
        new_test_ext().execute_with(|| {
            let url = b"http://example.com".to_vec();

            assert_ok!(MinerPallet::<Test>::add_to_whitelist(RuntimeOrigin::root(), url.clone()));

            let bounded_url: BoundedVec<u8, MaxUrlLength> = url.try_into().unwrap();
            assert!(Whitelist::<Test>::contains_key(&bounded_url));

            // Check emitted events
            let events = System::events();
            println!("Events: {:?}", events);
        });
    }

    #[test]
    fn remove_from_whitelist_works() {
        new_test_ext().execute_with(|| {
            let url = b"http://example.com".to_vec();
            let bounded_url: BoundedVec<u8, MaxUrlLength> = url.clone().try_into().unwrap();

            // Insert into Whitelist
            Whitelist::<Test>::insert(&bounded_url, ());
            assert!(Whitelist::<Test>::contains_key(&bounded_url));

            // Remove from Whitelist
            assert_ok!(MinerPallet::<Test>::remove_from_whitelist(RuntimeOrigin::root(), url));
            assert!(!Whitelist::<Test>::contains_key(&bounded_url));

            // Check emitted events
            let events = System::events();
            println!("Events: {:?}", events);
        });
    }
}
