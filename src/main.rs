use d3d12 as dx;
use raw_window_handle::HasRawWindowHandle;
use winapi::shared::dxgi::DXGI_ADAPTER_DESC;
use winapi::shared::winerror::*;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod os_helpers;
use os_helpers::hr_string;

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

fn print_adapters(factory: &d3d12::Factory4) {
    for i in 0.. {
        let (adapter, hr) = factory.enumerate_adapters(i);

        if SUCCEEDED(hr) {
            let mut desc: DXGI_ADAPTER_DESC;
            unsafe {
                desc = zeroed();
                adapter.GetDesc(&mut desc);
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

            println!("Adapter {}: {}", i, description);
            println!("    VendorId:      {:>10}", vendor_id);
            println!("    DeviceId:      {:>10}", device_id);
            println!("    SubSysId:      {:>10}", subsys_id);
            println!("    Revision:      {:>10}", revision);
            println!("    Video Memory:  {:>6} MiB", video_mem);
            println!("    System Memory: {:>6} MiB", system_mem);
            println!("    Shared Memory: {:>6} MiB", shared_mem);
            println!();
        } else if hr == DXGI_ERROR_NOT_FOUND {
            // Not found - we're at the last one.
            // Do not log this, it's expected
            break;
        } else {
            println!("Failed to enumerate adapter #{}: {}", i, hr_string(hr));
            break;
        }
    }
}

fn main() {
    let factory = check_hr!(d3d12::Factory4::create(dx::FactoryCreationFlags::DEBUG));

    // Log the adapters
    print_adapters(&factory);

    let _debug = check_hr!(d3d12::Debug::get_interface());
    let adapter = check_hr!(factory.enumerate_adapters(0));
    let _device = check_hr!(d3d12::Device::create(adapter, dx::FeatureLevel::L12_0));

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
