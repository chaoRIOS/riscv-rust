use std::os::raw::c_int;

#[link(name = "dramsim_interface")]
extern "C" {
	pub fn dramsim_test() -> c_int;
}

fn main() {
	unsafe {
		assert_eq!(0, dramsim_test() as i64);
	}
}

#[cfg(test)]
mod test {
	use std::os::raw::c_int;

	#[link(name = "cfoo")]
	extern "C" {
		pub fn add(a: c_int) -> c_int;
	}
	#[test]
	fn cadd() {
		unsafe {
			assert_eq!(1234, add(234 as c_int) as i64);
		}
	}
}
