#![cfg(test)]

use super::mock::*;

use runtime_io::with_externalities;
use srml_support::*;
use system::{self, EventRecord, Phase};

#[test]
fn set_ipns_id() {
    with_externalities(&mut initial_test_ext(), || {
        let current_block_number = 1000;
        System::set_block_number(current_block_number);

        let alice = alice_account();
        let identity = "alice".as_bytes().to_vec();
        let ttl = discovery::MINIMUM_LIFETIME + 100;
        assert!(Discovery::set_ipns_id(Origin::signed(alice), identity.clone(), Some(ttl)).is_ok());

        assert!(<discovery::AccountInfoByAccountId<Test>>::exists(&alice));
        let account_info = Discovery::account_info_by_account_id(&alice);
        assert_eq!(
            account_info,
            discovery::AccountInfo {
                identity: identity.clone(),
                expires_at: current_block_number + ttl
            }
        );

        assert_eq!(
            *System::events().last().unwrap(),
            EventRecord {
                phase: Phase::ApplyExtrinsic(0),
                event: MetaEvent::discovery(discovery::RawEvent::AccountInfoUpdated(
                    alice,
                    identity.clone()
                )),
            }
        );

        // Non role account trying to set account into should fail
        let bob = bob_account();
        assert!(Discovery::set_ipns_id(Origin::signed(bob), identity.clone(), None).is_err());
        assert!(!<discovery::AccountInfoByAccountId<Test>>::exists(&bob));
    });
}

#[test]
fn unset_ipns_id() {
    with_externalities(&mut initial_test_ext(), || {
        let alice = alice_account();

        <discovery::AccountInfoByAccountId<Test>>::insert(
            &alice,
            discovery::AccountInfo {
                expires_at: 1000,
                identity: "alice".as_bytes().to_vec(),
            },
        );

        assert!(<discovery::AccountInfoByAccountId<Test>>::exists(&alice));

        assert!(Discovery::unset_ipns_id(Origin::signed(alice)).is_ok());
        assert!(!<discovery::AccountInfoByAccountId<Test>>::exists(&alice));

        assert_eq!(
            *System::events().last().unwrap(),
            EventRecord {
                phase: Phase::ApplyExtrinsic(0),
                event: MetaEvent::discovery(discovery::RawEvent::AccountInfoRemoved(alice)),
            }
        );
    });
}

#[test]
fn is_account_info_expired() {
    with_externalities(&mut initial_test_ext(), || {
        let alice = alice_account();
        let expires_at = 1000;
        let id = "alice".as_bytes().to_vec();
        <discovery::AccountInfoByAccountId<Test>>::insert(
            &alice,
            discovery::AccountInfo {
                expires_at,
                identity: id.clone(),
            },
        );

        System::set_block_number(expires_at - 10);
        assert!(!Discovery::is_account_info_expired(&alice));

        System::set_block_number(expires_at + 10);
        assert!(Discovery::is_account_info_expired(&alice));
    });
}

#[test]
fn set_default_lifetime() {
    with_externalities(&mut initial_test_ext(), || {
        let lifetime = discovery::MINIMUM_LIFETIME + 2000;
        // priviliged method should fail if not from root origin
        assert!(
            Discovery::set_default_lifetime(Origin::signed(1), lifetime).is_err(),
            ""
        );
        assert!(
            Discovery::set_default_lifetime(Origin::ROOT, lifetime).is_ok(),
            ""
        );
        assert_eq!(Discovery::default_lifetime(), lifetime, "");

        // cannot set default lifetime to less than minimum
        assert!(
            Discovery::set_default_lifetime(Origin::ROOT, discovery::MINIMUM_LIFETIME - 1).is_err(),
            ""
        );
    });
}

#[test]
fn set_bootstrap_endpoints() {
    with_externalities(&mut initial_test_ext(), || {
        let endpoints = vec!["endpoint1".as_bytes().to_vec()];
        // priviliged method should fail if not from root origin
        assert!(
            Discovery::set_bootstrap_endpoints(Origin::signed(1), endpoints.clone()).is_err(),
            ""
        );
        assert!(
            Discovery::set_bootstrap_endpoints(Origin::ROOT, endpoints.clone()).is_ok(),
            ""
        );
        assert_eq!(Discovery::bootstrap_endpoints(), endpoints, "");
    });
}
