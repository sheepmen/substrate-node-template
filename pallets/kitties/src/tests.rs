use crate::mock::*;

#[test]
fn owned_kitties_can_append_value() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
    })
}