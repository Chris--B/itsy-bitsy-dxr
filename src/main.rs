use raw_window_handle::HasRawWindowHandle;
use winapi::shared::dxgi::DXGI_ADAPTER_DESC;
use winapi::shared::winerror::*;

use structopt::StructOpt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod os_helpers;
use os_helpers::hr_string;

use std::mem::zeroed;

const POSSIBLE_FEATURE_LEVELS: &[&str] = &[
    "9_1", "9_2", "9_3", "10_0", "10_1", "11_0", "11_1", "12_0", "12_1",
];

fn parse_feature_level(text: &str) -> Result<d3d12::FeatureLevel, String> {
    let text = text.trim();
    match text {
        "9_1" => Ok(d3d12::FeatureLevel::L9_1),
        "9_2" => Ok(d3d12::FeatureLevel::L9_2),
        "9_3" => Ok(d3d12::FeatureLevel::L9_3),
        "10_0" => Ok(d3d12::FeatureLevel::L10_0),
        "10_1" => Ok(d3d12::FeatureLevel::L10_1),
        "11_0" => Ok(d3d12::FeatureLevel::L11_0),
        "11_1" => Ok(d3d12::FeatureLevel::L11_1),
        "12_0" => Ok(d3d12::FeatureLevel::L12_0),
        "12_1" => Ok(d3d12::FeatureLevel::L12_1),
        // TODO: Update this for "DX12 Ultimate"
        // "12_2" => Ok(d3d12::FeatureLevel::L12_2),
        _ => Err("See MSDN for valid levels: https://docs.microsoft.com/en-us/windows/win32/direct3d12/hardware-feature-levels".to_string()),
    }
}

#[derive(StructOpt)]
struct Opts {
    /// Index of adapter to use
    #[structopt(short, long, default_value = "0")]
    adapter: u32,

    /// Use the warp adapter instead of an adapter index
    #[structopt(short, long, conflicts_with("adapter"))]
    warp: bool,

    /// DX Feature level to request
    #[structopt(
        short,
        long,
        default_value = "11_0",
        parse(try_from_str = parse_feature_level),
        possible_values=POSSIBLE_FEATURE_LEVELS
    )]
    feature_level: d3d12::FeatureLevel,
}

fn display_adapter(adapter: &d3d12::Adapter1, label: &str) {
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

    println!("{}: {}", label, description);
    println!("    VendorId:      {:>10}", vendor_id);
    println!("    DeviceId:      {:>10}", device_id);
    println!("    SubSysId:      {:>10}", subsys_id);
    println!("    Revision:      {:>10}", revision);
    println!("    Video Memory:  {:>6} MiB", video_mem);
    println!("    System Memory: {:>6} MiB", system_mem);
    println!("    Shared Memory: {:>6} MiB", shared_mem);
    println!();
}

fn main() {
    let opts = Opts::from_args();

    let factory = check_hr!(d3d12::Factory4::create(d3d12::FactoryCreationFlags::DEBUG));
    let _debug = check_hr!(d3d12::Debug::get_interface());

    // Select adapter
    let adapter: d3d12::Adapter1 = if opts.warp {
        println!("Using WARP adapter");

        check_hr! {
            unsafe {
                use winapi::shared::dxgi::*;
                use winapi::Interface;

                let mut warp_adapter = d3d12::Adapter1::null();
                let hr = factory.EnumWarpAdapter(&IDXGIAdapter1::uuidof(), warp_adapter.mut_void());

                (warp_adapter, hr)
            }
        }
    } else {
        check_hr!(factory.enumerate_adapters(opts.adapter))
    };

    // Log the adapters
    for i in 0.. {
        let (this_adapter, hr) = factory.enumerate_adapters(i);

        if SUCCEEDED(hr) {
            // If we're choosing an adapter, label it to make it clear what we're using
            // This will be omitted when using the warp adapter
            let icon = if !opts.warp && opts.adapter == i {
                "✨"
            } else {
                " "
            };
            let label = format!("{} Adapter {}:", icon, i);

            display_adapter(&this_adapter, &label);
        } else if hr == DXGI_ERROR_NOT_FOUND {
            // Not found - we're at the last one.
            // Do not log this, it's expected
            break;
        } else {
            println!("Failed to enumerate adapter #{}: {}", i, hr_string(hr));
            break;
        }
    }

    let _device = check_hr!(d3d12::Device::create(adapter, opts.feature_level));

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
