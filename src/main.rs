use std::ffi::CStr;
use std::fmt::Display;
use std::mem;
use std::ptr;

use itertools::Itertools;

use serde::Serialize;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D9::*;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::UI::WindowsAndMessaging::GetShellWindow;

mod formats;
mod guids;

use formats::*;
use guids::*;

#[derive(Debug, Serialize)]
struct Adapter {
    pub index: u32,
    pub driver: String,
    pub description: String,
    pub device_name: String,
    pub driver_version: i64,
    pub vendor_id: u32,
    pub device_id: u32,
    pub sub_system_id: u32,
    pub revision: u32,
    pub device_identifier: DisplayGuid,
    pub whql_level: u32,
}

#[derive(Debug)]
struct AdapterCapabilities {
    adapter: Adapter,
    results: Vec<TestResult>,
}

#[derive(Debug)]
struct TestResult {
    codec: NamedGuid,
    format: NamedFormat,
    resolution: (u32, u32),
    result: String,
}

impl Adapter {
    unsafe fn from_c(index: u32, adapter: &D3DADAPTER_IDENTIFIER9) -> Self {
        Adapter {
            index,
            driver: convert_string(&adapter.Driver[..]),
            description: convert_string(&adapter.Description[..]),
            device_name: convert_string(&adapter.DeviceName[..]),
            driver_version: adapter.DriverVersion,
            vendor_id: adapter.VendorId,
            device_id: adapter.DeviceId,
            sub_system_id: adapter.SubSysId,
            revision: adapter.Revision,
            device_identifier: DisplayGuid(adapter.DeviceIdentifier),
            whql_level: adapter.WHQLLevel,
        }
    }
}

const RESOLUTIONS: [(u32, u32); 2] = [(1920, 1080), (3840, 2160)];

fn main() {
    let mut d3d = unsafe { create_d3d() };

    let adapters = unsafe { list_adapters(&mut d3d).unwrap() };

    let adapters = adapters
        .into_iter()
        .unique_by(|a| a.device_identifier.to_string())
        .collect::<Vec<_>>();

    dbg!(&adapters);

    for adapter in adapters {
        unsafe {
            let adapter_name = adapter.description.clone();

            match test_hardware_support(&mut d3d, adapter) {
                Ok(caps) => {
                    println!("{}:", caps.adapter.description);
                    for r in caps.results {
                        println!(
                            "  {} {}x{}@{}: {}",
                            r.codec, r.resolution.0, r.resolution.1, r.format, r.result
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to test HW support for {:?}: {}", adapter_name, e);
                }
            }
        }
    }

    // let hr = device_manager.reset(d3d_device, token);
}

unsafe fn test_hardware_support(
    d3d: &mut IDirect3D9Ex,
    adapter: Adapter,
) -> anyhow::Result<AdapterCapabilities> {
    let d3d_device = create_d3d_device(d3d, &adapter)?;

    let mut token = 0;
    let mut device_manager = None;
    unsafe {
        DXVA2CreateDirect3DDeviceManager9(&mut token, &mut device_manager)?;
    }

    let device_manager = device_manager.unwrap();
    device_manager.ResetDevice(&d3d_device, token)?;

    let mut video_decoder_service: Option<IDirectXVideoDecoderService> = None;
    let device = device_manager.OpenDeviceHandle()?;
    device_manager.GetVideoService(
        device,
        &IDirectXVideoDecoderService::IID,
        &mut video_decoder_service as *mut _ as *mut *mut _,
    )?;
    device_manager.CloseDeviceHandle(device)?;

    let video_decoder_service = video_decoder_service.unwrap();
    let mut input_count = 0;
    let mut input_list = ptr::null_mut();

    video_decoder_service.GetDecoderDeviceGuids(&mut input_count, &mut input_list)?;

    let mut results = Vec::new();
    for i in 0..input_count {
        let guid = *input_list.offset(i as _);

        if let Some((guid, name)) = ALL_GUIDS.iter().find(|(g, _)| g == &guid) {
            // println!("  {name}");

            let formats = get_decoder_formats(&video_decoder_service, guid)?;

            for format in formats {
                for resolution in RESOLUTIONS {
                    let result = test_format_capabilities(
                        &video_decoder_service,
                        *guid,
                        format,
                        resolution.0,
                        resolution.1,
                    );

                    results.push(TestResult {
                        codec: NamedGuid::new(*guid),
                        format: NamedFormat::new(format),
                        resolution,
                        result: match result {
                            Ok(_) => "OK".into(),
                            Err(e) => e.to_string()
                        },
                    });
                }
            }
        } else {
            // println!("  {:?}", guid);
        }
    }

    Ok(AdapterCapabilities { adapter, results })
}

unsafe fn get_decoder_formats(
    vds: &IDirectXVideoDecoderService,
    codec: &GUID,
) -> anyhow::Result<Vec<D3DFORMAT>> {
    let mut format_count = 0;
    let mut format_list = ptr::null_mut();

    vds.GetDecoderRenderTargets(codec, &mut format_count, &mut format_list)?;

    let mut formats = Vec::new();
    for i in 0..format_count {
        let format = *format_list.offset(i as _);
        formats.push(format);
    }

    Ok(formats)
}

unsafe fn test_format_capabilities(
    vds: &IDirectXVideoDecoderService,
    codec: GUID,
    format: D3DFORMAT,
    width: u32,
    height: u32,
) -> anyhow::Result<()> {
    let mut desc: DXVA2_VideoDesc = mem::zeroed();
    desc.SampleWidth = width;
    desc.SampleHeight = height;
    desc.Format = format;

    let decoder_config = get_decoder_configuration(vds, codec, &desc)?;

    let surface_width = round_up_to_multiple_of(width, 128);
    let surface_height = round_up_to_multiple_of(height, 128);
    desc.SampleWidth = surface_width;
    desc.SampleHeight = surface_height;

    // dbg!(surface_width, surface_height);
    let surface_count = 20;
    let mut surface_list: Vec<Option<IDirect3DSurface9>> = vec![None; 64];

    vds.CreateSurface(
        surface_width,
        surface_height,
        surface_count - 1,
        format,
        D3DPOOL_DEFAULT,
        0,
        DXVA2_VideoDecoderRenderTarget,
        surface_list.as_mut_ptr(),
        None,
    )?;

    let decoder = vds.CreateVideoDecoder(
        &codec,
        &desc,
        &decoder_config,
        &surface_list[..surface_count as usize],
    )?;

    Ok(())
}

unsafe fn get_decoder_configuration(
    vds: &IDirectXVideoDecoderService,
    codec: GUID,
    desc: &DXVA2_VideoDesc,
) -> anyhow::Result<DXVA2_ConfigPictureDecode> {
    let mut cfg_count = 0;
    let mut cfg_list = ptr::null_mut();

    let mut best_cfg = None;

    vds.GetDecoderConfigurations(&codec, desc, None, &mut cfg_count, &mut cfg_list)?;

    for i in 0..cfg_count {
        let cfg = *cfg_list.offset(i as _);

        // dbg!(&cfg);
        if cfg.ConfigBitstreamRaw == 1 || cfg.ConfigBitstreamRaw == 2 {
            best_cfg = Some(cfg);
        }
    }

    if let Some(cfg) = best_cfg {
        return Ok(cfg);
    }

    anyhow::bail!("Failed to find any decoder configuration");
}

unsafe fn create_d3d() -> IDirect3D9Ex {
    let d3d_object = Direct3DCreate9Ex(D3D_SDK_VERSION).unwrap();

    d3d_object
}

unsafe fn create_d3d_device(
    d3d: &mut IDirect3D9Ex,
    adapter: &Adapter,
) -> anyhow::Result<IDirect3DDevice9> {
    let mut d3ddm: D3DDISPLAYMODEEX = mem::zeroed();
    d3ddm.Size = mem::size_of::<D3DDISPLAYMODEEX>() as _;

    d3d.GetAdapterDisplayModeEx(adapter.index, &mut d3ddm, ptr::null_mut())?;

    let mut params: D3DPRESENT_PARAMETERS = mem::zeroed();
    params.Flags = D3DPRESENTFLAG_VIDEO;
    params.Windowed = TRUE;
    params.SwapEffect = D3DSWAPEFFECT_DISCARD;

    params.BackBufferWidth = 640;
    params.BackBufferHeight = 480;
    params.BackBufferFormat = d3ddm.Format;

    let mut device = None;
    d3d.CreateDevice(
        adapter.index,
        D3DDEVTYPE_HAL,
        GetShellWindow(),
        D3DCREATE_SOFTWARE_VERTEXPROCESSING as u32
            | D3DCREATE_MULTITHREADED as u32
            | D3DCREATE_FPU_PRESERVE as u32,
        &mut params as _,
        &mut device,
    )?;

    Ok(device.unwrap())
}

unsafe fn list_adapters(d3d_object: &mut IDirect3D9Ex) -> anyhow::Result<Vec<Adapter>> {
    let adapter_count = d3d_object.GetAdapterCount();

    let mut adapters = Vec::new();

    let mut adapter: D3DADAPTER_IDENTIFIER9 = mem::zeroed();
    for i in 0..adapter_count {
        d3d_object.GetAdapterIdentifier(i, 0, &mut adapter)?;

        adapters.push(Adapter::from_c(i, &adapter));
    }

    Ok(adapters)
}

fn convert_string(bytes: &[u8]) -> String {
    let cstr = unsafe { CStr::from_ptr(bytes.as_ptr() as _) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

fn round_up_to_multiple_of(num: u32, mult: u32) -> u32 {
    ((num + (mult - 1)) / mult) * mult
}
