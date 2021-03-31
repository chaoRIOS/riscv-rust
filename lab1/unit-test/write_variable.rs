pub const ID2IS_LEN: usize = 38;

fn main() {
	let mut arr: [u8; (ID2IS_LEN / 8) as usize + 1] = [0, 0, 0, 0, 0];
	println!("{:?}", arr);
	println!("Unit test of write_variable");
	println!("case 1");
	write_variable(5, 1, 2, &mut arr);
	println!("{:?}", arr);
	println!("case 2");
	write_variable(65533, 10, 3, &mut arr);
	println!("{:?}", arr);
	println!("case 3");
	write_variable(1048023161, 29, 0, &mut arr);
	println!("{:?}", arr);
	/*
		0 1 2 3 4 5 6 7 | 8 9 10 11 12 13 14 15 | 16 17 18 19 20 21 22 23 | 24 25 26 27 28 29 30 31 | 32 33 34 35 36 37 38 39 | ...
	ca1                   1 x  x
						  1
	ca2                     1 0  1  1  1  1  1    1   1  1 x  x
						  251                     7
	ca3             1 0   0 1 1  1  1  0  0  0    0   0  1  0  0  1  1  1   1  0  1  1  1  0  0  1    1  1  1  x
		64                30                      228                       157                       7
	*/
}

pub const BYTE_ARRAY_OFFSET: usize = 6; // this is a const, e.g. 320 array write 314 flush to right, make it be 6
										/*
										input:
										@value : the value to be written to byte_array
										@width : [0...width-1] bit of value is to be written to byte_array
										@offset: index of (offset + BYTE_ARRAY_OFFSET)-th bit will be the first bit of byte_array to be written
										@byte_array : the array to be written
										*/
pub fn write_variable(
	value: u64,
	width: usize,
	offset: usize, // consider this is 0--0ffset-1 used, offset+1 unused
	byte_array: &mut [u8; (ID2IS_LEN / 8) as usize + 1],
) {
	let target_l = (BYTE_ARRAY_OFFSET + offset) / 8 as usize;
	let target_l_width = 8 - (BYTE_ARRAY_OFFSET + offset) % 8;
	let target_r = (BYTE_ARRAY_OFFSET + offset + width) / 8 as usize;
	let target_r_width = (BYTE_ARRAY_OFFSET + offset + width) % 8;
	let mut mask: u8;
	let mut bit_processed = target_l_width;
	let mut value_byte: u8;
	let flitered_value = value & (make_all_ones(width) as u64);
	println!(
		"debuglog:: at write_variable, flitered_value={0}",
		flitered_value
	);
	{
		mask = (make_all_ones(target_l_width) << (8 - target_l_width)) as u8;
		value_byte =
			((flitered_value & make_all_ones(target_l_width)) << (8 - target_l_width)) as u8;
		println!(
			"debuglog:: Processing leftwidth, tar_l_wid={0},tar_l={1},mask={2},valuebyte={3}",
			target_l_width, target_l, mask, value_byte
		);
		write_byte(value_byte, mask, &mut byte_array[target_l]);
	}
	for i in target_l + 1..target_r {
		mask = make_all_ones(8) as u8;
		value_byte = get_range_bits(flitered_value, bit_processed, bit_processed + 8) as u8;
		println!(
			"debuglog:: Processing midwidth, i={0},mask={1},valuebyte={2}",
			i, mask, value_byte
		);
		write_byte(value_byte, mask, &mut byte_array[i]);
		bit_processed += 8;
	}
	if target_l < target_r {
		mask = make_all_ones(target_r_width) as u8;
		value_byte = (get_range_bits(flitered_value, bit_processed, width)
			& make_all_ones(target_r_width)) as u8;
		println!("debuglog:: Processing rightwidth, tar_r_wid={0},tar_r={1},mask={2},valuebyte={3},bitprocessed={4},getrangebit={5}",target_r_width,target_r,mask,value_byte,bit_processed,get_range_bits(flitered_value, bit_processed, width));
		write_byte(value_byte, mask, &mut byte_array[target_r]);
	}
}

fn write_byte(value: u8, mask: u8, tar: &mut u8) {
	*tar = (value & mask) | ((*tar) & (!mask));
}

// note : return [left_side, right_side) bits of value, index for bit start from 0
fn get_range_bits(value: u64, left_side: usize, right_side: usize) -> u64 {
	println!(
		"debuglog:: get_range_bits: value={0},left={1},rig={2}",
		value, left_side, right_side
	);
	(value >> left_side) & make_all_ones(right_side - left_side)
}

fn make_all_ones(width: usize) -> u64 {
	(1 << width) - 1
}
