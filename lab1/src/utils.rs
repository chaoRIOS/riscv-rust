use crate::pkg::ID2IS_LEN;
pub const BYTE_ARRAY_OFFSET: usize = 0;

/// input:
/// @value : the value to be written to byte_array
/// @width : [0...width-1] bit of value is to be written to byte_array
/// @offset: index of (offset + BYTE_ARRAY_OFFSET)-th bit will be the first bit of byte_array to be written
/// @byte_array : the array to be written
pub fn write_variable(
	value: u64,
	width: usize,
	offset: usize, // consider this is 0--0ffset-1 used, offset+1 unused
	byte_array: &mut [u8; (ID2IS_LEN / 8) as usize + 1],
) {
	#[cfg(debug_assertions)]
	println!(
		"[RS] Writing {} with {} bits to {} offset",
		value, width, offset
	);
	let target_l = (BYTE_ARRAY_OFFSET + offset) / 8 as usize;
	let target_l_width = 8 - (BYTE_ARRAY_OFFSET + offset) % 8;
	let target_r = (BYTE_ARRAY_OFFSET + offset + width) / 8 as usize;
	let target_r_width = (BYTE_ARRAY_OFFSET + offset + width) % 8;
	let mut mask: u8;
	let mut bit_processed = target_l_width;
	let mut value_byte: u8;
	let flitered_value = value & (make_all_ones(width) as u64);
	#[cfg(debug_assertions)]
	println!(
		"debuglog:: at write_variable, flitered_value={0}",
		flitered_value
	);
	if target_l == target_r {
		let rmask = make_all_ones(target_r_width) as u8;
		let lmask = (make_all_ones(target_l_width) << (8 - target_l_width)) as u8;
		value_byte = ((value << (8 - target_l_width)) & make_all_ones(8)) as u8;
		#[cfg(debug_assertions)]
		println!(
			"debuglog:: target-l-width={0},valuebyte={1}",
			target_l_width, value
		);
		write_byte(value_byte, rmask & lmask, &mut byte_array[target_l]);
		return;
	}
	{
		mask = (make_all_ones(target_l_width) << (8 - target_l_width)) as u8;
		value_byte =
			((flitered_value & make_all_ones(target_l_width)) << (8 - target_l_width)) as u8;
		#[cfg(debug_assertions)]
		println!(
			"debuglog:: Processing leftwidth, tar_l_wid={0},tar_l={1},mask={2},valuebyte={3}",
			target_l_width, target_l, mask, value_byte
		);
		write_byte(value_byte, mask, &mut byte_array[target_l]);
	}
	for i in target_l + 1..target_r {
		mask = make_all_ones(8) as u8;
		value_byte = get_range_bits(flitered_value, bit_processed, bit_processed + 8) as u8;
		#[cfg(debug_assertions)]
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
		#[cfg(debug_assertions)]
		println!(
			"debuglog:: Processing rightwidth, tar_r_wid={0},tar_r={1},mask={2},valuebyte={3},bitprocessed={4},getrangebit={5}",
			target_r_width,
			target_r,
			mask,
			value_byte,
			bit_processed,
			get_range_bits(flitered_value, bit_processed, width));
		match target_r_width {
			0 => {}
			_ => {
				write_byte(value_byte, mask, &mut byte_array[target_r]);
			}
		}
	}
}

fn write_byte(value: u8, mask: u8, tar: &mut u8) {
	*tar = (value & mask) | ((*tar) & (!mask));
}

// note : return [left_side, right_side) bits of value, index for bit start from 0
fn get_range_bits(value: u64, left_side: usize, right_side: usize) -> u64 {
	#[cfg(debug_assertions)]
	println!(
		"debuglog:: get_range_bits: value={0},left={1},right={2}",
		value, left_side, right_side
	);
	match left_side < 64 {
		true => (value >> left_side) & make_all_ones(right_side - left_side),
		false => 0 as u64,
	}
}

fn make_all_ones(width: usize) -> u64 {
	match width <= 63 {
		true => (1 << width) - 1,
		false => 0xffff_ffff_ffff_ffff,
	}
}

/// Slice byte_array [left_index,width+left_index], return as a u64
/// An inversion function of write_variable
pub fn read_variable(
	left_index: usize,
	width: usize,
	byte_array: &mut [u8; (ID2IS_LEN / 8) as usize + 1],
) -> u64 {
	if width > 64 {
		println!("[Err]: Width out of range, maximium 64");
	}
	let target_l = (BYTE_ARRAY_OFFSET + left_index) / 8 as usize;
	let target_l_width = 8 - (BYTE_ARRAY_OFFSET + left_index) % 8;
	let target_r = (BYTE_ARRAY_OFFSET + left_index + width) / 8 as usize;
	let target_r_width = (BYTE_ARRAY_OFFSET + left_index + width) % 8;
	let mut mask: u8;
	let mut value_byte: u8;
	let mut value: u64 = 0;
	if target_l == target_r {
		let rmask = make_all_ones(target_r_width) as u8;
		let lmask = (make_all_ones(target_l_width) << (8 - target_l_width)) as u8;
		return read_byte(rmask & lmask, byte_array[target_l]) as u64 >> (8 - target_l_width);
	}

	{
		mask = make_all_ones(target_r_width) as u8;
		value_byte = read_byte(mask, byte_array[target_r]);
		value = value_byte as u64;
		#[cfg(debug_assertions)]
		println!(
			"debuglog:: processing rightwidth, valuebyte={0},value={1},bytearray[]={2},mask={3}",
			value_byte, value, byte_array[target_r], mask
		);
	}

	let mut i: usize = target_r - 1;
	while i > target_l {
		mask = make_all_ones(8) as u8;
		value_byte = read_byte(mask, byte_array[i]);
		value = (value << 8) | value_byte as u64;
		#[cfg(debug_assertions)]
		println!(
			"debuglog:: Processing midwidth, i={0},mask={1},valuebyte={2},value={3}",
			i, mask, value_byte, value
		);
		i = i - 1;
	}

	if target_l < target_r {
		mask = (make_all_ones(target_l_width) << (8 - target_l_width)) as u8;
		value_byte = read_byte(mask, byte_array[target_l]) >> (8 - target_l_width) as u8;
		value = (value << target_l_width) | value_byte as u64;
		#[cfg(debug_assertions)]
		println!(
			"debuglog:: Processing leftwidth, valuebyte={0},value={1}",
			value_byte, value
		);
	}
	value
}

fn read_byte(mask: u8, tar: u8) -> u8 {
	tar & mask
}

#[cfg(test)]
mod tests {
	use crate::unittest::*;

	#[test]
	fn test_write_variable() {}

	// @TODO: add test_read_variable
}
