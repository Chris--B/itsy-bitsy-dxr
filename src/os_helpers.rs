use winapi::shared::winerror::*;

#[macro_export]
macro_rules! check_hr {
    ($call:expr) => {{
        let (obj, _hr) = crate::check_hr2!($call);
        obj
    }};
}

#[macro_export]
macro_rules! check_hr2 {
    ($call:expr) => {{
        let (obj, hr): (_, d3d12::HRESULT) = $call;
        if SUCCEEDED(hr) {
            // Log nothing, we're good :)
        } else {
            let location = format!("{}:{}", file!(), line!());
            println!(
                "{location}: {call}:\n    {hr}",
                location = location,
                hr = crate::os_helpers::hr_string(hr),
                call = stringify!($call)
            );
        }
        (obj, hr)
    }};
}

/// Turn an HRESULT code into something Google-able
pub fn hr_string(hr: d3d12::HRESULT) -> String {
    let mut buffer = [0u8; 128];

    // Query the system message for this result code.
    unsafe {
        use std::ptr::{null, null_mut};
        use winapi::shared::ntdef::{LANG_NEUTRAL, MAKELANGID, SUBLANG_DEFAULT};
        use winapi::um::winbase::{FormatMessageA, FORMAT_MESSAGE_FROM_SYSTEM};

        FormatMessageA(
            FORMAT_MESSAGE_FROM_SYSTEM,
            null(),
            hr as u32,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT).into(), // Default language
            buffer.as_mut_ptr() as *mut i8,
            buffer.len() as u32,
            null_mut(),
        );
    }

    let success_icon = if SUCCEEDED(hr) { "✔️" } else { "❌" };

    let full_msg = std::str::from_utf8(&buffer).expect("Invalid utf8");
    let str_len = full_msg
        .trim_end_matches(char::from(0)) // Trailing NULs
        .trim_end() // Trailing whitespace (FormatMessage may add newlines)
        .len();
    format!("{} (0x{:08X}) - {}", success_icon, hr, &full_msg[..str_len])
}
