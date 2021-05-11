use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[link(name = "interface")]
extern "C" {
	pub fn Setup(rqst_to_memory: *const c_char, resp_to_cpu: *const c_char) -> c_int;
	pub fn SendRqst(trace_str: *const c_char) -> c_int;
	pub fn RecvResp() -> c_int;
	pub fn RecvRespString() -> *const c_char;
	pub fn Terminate();
}

pub fn setup_pipe(request_pipe: &str, response_pipe: &str) -> i32 {
	let req = CString::new(request_pipe).unwrap();
	let resp = CString::new(response_pipe).unwrap();

	unsafe {
		Setup(
			request_pipe.as_ptr() as *const c_char,
			response_pipe.as_ptr() as *const c_char,
		)
	}
}

fn send_request(trace_str: &str) -> i32 {
	let c_trace_str = CString::new(trace_str).unwrap();
	unsafe { SendRqst(c_trace_str.as_ptr() as *const c_char) }
}

fn recieve_response() -> i32 {
	unsafe { RecvResp() }
}

fn recieve_response_string() -> String {
	unsafe {
		CStr::from_ptr(RecvRespString())
			.to_string_lossy()
			.into_owned()
	}
}

pub fn terminate_pipe() {
	unsafe { Terminate() }
}

// fn main() -> std::io::Result<()> {
// 	let rqst_path = CString::new("/home/cwang/work/riscv-rust/lab2/rqst_to_memory").unwrap();
// 	let resp_path = CString::new("/home/cwang/work/riscv-rust/lab2/resp_to_cpu").unwrap();
// 	unsafe {
// 		Setup(
// 			rqst_path.as_ptr() as *const c_char,
// 			resp_path.as_ptr() as *const c_char,
// 		);
// 	}

// 	send_request("0000000083000000 READ 100");
// 	send_request("0000000083000000 READ 200");
// 	send_request("0000000083000000 READ 300");
// 	// println!("Sent!");
// 	for i in 0..3 {
// 		let mut resp = recieve_response();
// 		while resp < 0 {
// 			resp = recieve_response();
// 		}
// 		println!("Recieving {}", recieve_response_string().as_str());
// 	}

// 	unsafe {
// 		Terminate();
// 	}
// 	Ok(())
// }
