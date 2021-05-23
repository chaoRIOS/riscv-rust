use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[link(name = "interface", kind = "static")]
extern "C" {
	pub fn Setup(rqst_to_memory: *const c_char, resp_to_cpu: *const c_char) -> c_int;
	pub fn SendRqst(trace_str: *const c_char) -> c_int;
	pub fn RecvResp() -> c_int;
	pub fn RecvRespString() -> *const c_char;
	pub fn Terminate();
}

// Safe C ffi interfaces

/// Setup linux fifo
pub fn setup_pipe(request_pipe: &str, response_pipe: &str) -> i32 {
	let req = CString::new(request_pipe).unwrap();
	let resp = CString::new(response_pipe).unwrap();

	unsafe {
		Setup(
			req.as_ptr() as *const c_char,
			resp.as_ptr() as *const c_char,
		)
	}
}

/// Send a trace to pipe
pub fn send_request(trace_str: &str) -> i32 {
	let c_trace_str = CString::new(trace_str).unwrap();
	unsafe { SendRqst(c_trace_str.as_ptr() as *const c_char) }
}

/// Check response in pipe
fn recieve_response() -> i32 {
	unsafe { RecvResp() }
}

/// Resolve response
fn recieve_response_string() -> String {
	unsafe {
		CStr::from_ptr(RecvRespString())
			.to_string_lossy()
			.into_owned()
	}
}

/// Poll the pipe until reseponse available
pub fn get_response() -> String {
	// Poll
	loop {
		if recieve_response() > 0 {
			break;
		}
	}

	// Resolve
	recieve_response_string()
}

/// Terminate pipe
pub fn terminate_pipe() {
	unsafe { Terminate() }
}
