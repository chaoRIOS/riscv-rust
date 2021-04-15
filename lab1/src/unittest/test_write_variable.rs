pub fn test_write_variable() {
	use crate::pkg::ID2IS_LEN;
	use crate::utils::write_variable;
	let mut arr: [u8; (ID2IS_LEN / 8) as usize + 1] = [0; (ID2IS_LEN / 8) as usize + 1];
	write_variable(5, 1, 2, &mut arr);
	assert_eq!(
		arr,
		[
			4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
		]
	);
	write_variable(65533, 10, 3, &mut arr);
	assert_eq!(
		arr,
		[
			0xec, 0x1f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
		]
	);
	write_variable(0xffff_ffff_ffff_ffff, 64, 0, &mut arr);
	assert_eq!(
		arr,
		[
			0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
		]
	);
}
