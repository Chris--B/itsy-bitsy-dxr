use d3d12 as dx;
use dx::{D3D12Lib, DxgiLib};
use raw_window_handle::HasRawWindowHandle;
use winapi::shared::dxgi::DXGI_ADAPTER_DESC;
use winapi::shared::winerror::*;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::mem::zeroed;

#[allow(dead_code)]
struct Opt {
    /// Index of adapter to use
    adapter: Option<usize>,

    /// Use the warp adapter instead
    warp: bool,

    /// DX Feature level to request
    feature_level: dx::FeatureLevel,
}

type LazyResult<T> = Result<T, Box<dyn std::error::Error>>;

macro_rules! check_hr {
    ($call:expr) => {{
        let (obj, _hr) = check_hr2!($call);
        obj
    }};
}

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
                hr = hr_string(hr),
                call = stringify!($call)
            );
        }
        (obj, hr)
    }};
}

/// Turn an HRESULT code into something Google-able
fn hr_string(hr: d3d12::HRESULT) -> String {
    let mut buffer = [0u8; 1024];

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

fn main() -> LazyResult<()> {
    let d3d12 = D3D12Lib::new()?;
    let _debug_ctl = check_hr!(d3d12.get_debug_interface()?);

    let dxgi: DxgiLib = DxgiLib::new()?;
    let factory = check_hr!(dxgi.create_factory2(dx::FactoryCreationFlags::DEBUG)?);

    let mut adapter = None;

    for i in 0.. {
        let (this_adapter, hr) = factory.enumerate_adapters(i);

        if SUCCEEDED(hr) {
            let mut desc: DXGI_ADAPTER_DESC;
            unsafe {
                desc = zeroed();
                this_adapter.GetDesc(&mut desc);
            }

            let description = String::from_utf16(&desc.Description).unwrap();
            let vendor_id = format!("0x{:x}", desc.VendorId);
            let device_id = format!("0x{:x}", desc.DeviceId);
            let subsys_id = format!("0x{:x}", desc.SubSysId);
            let revision = format!("0x{:x}", desc.Revision);

            #[allow(non_upper_case_globals)]
            const MiB: usize = 1024 * 1024;
            let video_mem = desc.DedicatedVideoMemory / MiB;
            let system_mem = desc.DedicatedSystemMemory / MiB;
            let shared_mem = desc.SharedSystemMemory / MiB;

            let icon: &str;
            if adapter.is_none() {
                // Choose the first adapter that we find
                adapter = Some(this_adapter);
                // Label our chosen adapter
                icon = "✨";
            } else {
                icon = " ";
            }

            println!("{} Adapter {}: {}", icon, i, description);
            println!("    VendorId:      {:>10}", vendor_id);
            println!("    DeviceId:      {:>10}", device_id);
            println!("    SubSysId:      {:>10}", subsys_id);
            println!("    Revision:      {:>10}", revision);
            println!("    Video Memory:  {:>6} MiB", video_mem);
            println!("    System Memory: {:>6} MiB", system_mem);
            println!("    Shared Memory: {:>6} MiB", shared_mem);
            println!();
        } else if hr == DXGI_ERROR_NOT_FOUND as i32 {
            // Not found - we're at the last one.
            // Do not log this, it's expected
            break;
        } else {
            println!("Failed to enumerate adapter #{}: {}", i, hr_string(hr));
            break;
        }
    }
    let adapter = adapter.unwrap();
    let device = check_hr!(d3d12.create_device(adapter, dx::FeatureLevel::L12_0)?);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("☀ Itsy Bitsy DXR ☀")
        .build(&event_loop)
        .expect("Failed to create a window");

    let _h_wnd = window.raw_window_handle();

    event_loop.run(move |event, _, control_flow| {
        // *control_flow = ControlFlow::Wait;
        *control_flow = ControlFlow::Exit;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
