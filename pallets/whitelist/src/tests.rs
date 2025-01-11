#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pallet as WhitelistPallet;
    use crate::mock::{new_test_ext, RuntimeOrigin, RuntimeEvent, MaxUrlLength, System,};
    use crate::mock::Test;
    use crate::Whitelist;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok, BoundedVec};
    use sp_core::H256;
    use sp_runtime::AccountId32;
    use sp_runtime::traits::ConstU32;
    use sp_runtime::DispatchError;


    #[test]
    fn test_is_whitelisted() {
        new_test_ext().execute_with(|| {
            let valid_url = b"http://example.com".to_vec();
            let long_url = vec![b'a'; 300]; // Exceeds MaxUrlLength
            let invalid_url = b"http://not-in-whitelist.com".to_vec();

            // Add a valid URL to the whitelist
            assert_ok!(WhitelistPallet::<Test>::add_url(
                RuntimeOrigin::root(),
                valid_url.clone()
            ));

            // Test: The valid URL is whitelisted
            let result = WhitelistPallet::<Test>::is_whitelisted(valid_url.clone());
            assert!(result.is_ok() && result.unwrap(), "Expected URL to be whitelisted");

            // Test: An unwhitelisted URL is not whitelisted
            let result = WhitelistPallet::<Test>::is_whitelisted(invalid_url.clone());
            assert!(result.is_ok() && !result.unwrap(), "Expected URL to not be whitelisted");

            // Test: A URL that exceeds the max length should return an error
            let result = WhitelistPallet::<Test>::is_whitelisted(long_url.clone());
            assert!(matches!(result, Err(Error::<Test>::UrlTooLong)), "Expected UrlTooLong error");
        });
    }

    #[test]
    fn add_url_to_whitelist_works() {
        new_test_ext().execute_with(|| {
            let url = b"http://example.com".to_vec();
            let bounded_url: BoundedVec<u8, ConstU32<256>> = url.clone().try_into().unwrap();

            // Add the URL to the whitelist
            assert_ok!(WhitelistPallet::<Test>::add_url(
                RuntimeOrigin::root(),
                url.clone()
            ));

            // Verify storage
            assert!(Whitelist::<Test>::contains_key(&bounded_url));

            // Check events
            let events = System::events();
            assert!(events.iter().any(|record| matches!(
                record.event,
                RuntimeEvent::Whitelist(crate::Event::UrlAdded { ref url })
                if *url == *bounded_url
            )));
        });
    }

    #[test]
    fn add_url_exceeding_max_length_fails() {
        new_test_ext().execute_with(|| {
            let long_url = vec![b'a'; 300]; // Longer than MaxUrlLength
            assert_noop!(
                WhitelistPallet::<Test>::add_url(RuntimeOrigin::root(), long_url),
                Error::<Test>::UrlTooLong
            );
        });
    }

    #[test]
    fn add_duplicate_url_fails() {
        new_test_ext().execute_with(|| {
            let url = b"http://example.com".to_vec();
            let bounded_url: BoundedVec<u8, ConstU32<256>> = url.clone().try_into().unwrap();

            // Add URL
            assert_ok!(WhitelistPallet::<Test>::add_url(RuntimeOrigin::root(), url.clone()));

            // Try adding the same URL again
            assert_noop!(
                WhitelistPallet::<Test>::add_url(RuntimeOrigin::root(), url.clone()),
                Error::<Test>::UrlAlreadyWhitelisted
            );
        });
    }

    #[test]
    fn remove_url_from_whitelist_works() {
        new_test_ext().execute_with(|| {
            let url = b"http://example.com".to_vec();
            let bounded_url: BoundedVec<u8, ConstU32<256>> = url.clone().try_into().unwrap();

            // Add URL first
            Whitelist::<Test>::insert(&bounded_url, ());

            // Remove the URL
            assert_ok!(WhitelistPallet::<Test>::remove_url(RuntimeOrigin::root(), url.clone()));

            // Verify storage
            assert!(!Whitelist::<Test>::contains_key(&bounded_url));

            // Check events
            let events = System::events();
            assert!(events.iter().any(|record| matches!(
                record.event,
                RuntimeEvent::Whitelist(crate::Event::UrlRemoved { ref url })
                if *url == *bounded_url
            )));
        });
    }

    #[test]
    fn remove_nonexistent_url_fails() {
        new_test_ext().execute_with(|| {
            let url = b"http://example.com".to_vec();
            assert_noop!(
                WhitelistPallet::<Test>::remove_url(RuntimeOrigin::root(), url),
                Error::<Test>::UrlNotWhitelisted
            );
        });
    }

    #[test]
    fn non_root_cannot_add_url() {
        new_test_ext().execute_with(|| {
            let non_root = AccountId32::new([2; 32]);
            let url = b"http://example.com".to_vec();
            assert_noop!(
                WhitelistPallet::<Test>::add_url(RuntimeOrigin::signed(non_root), url),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }
}
