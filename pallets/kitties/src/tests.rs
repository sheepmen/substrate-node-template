use crate::mock::*;
use super::*;

#[test]
fn create_kitty_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(KittiesTest::create(Origin::signed(1)), Ok(()));
        let lock_event = TestEvent::kitties_event(RawEvent::Created(1, 0));
        assert!(System::events().iter().any(|a| a.event == lock_event));
    })
}