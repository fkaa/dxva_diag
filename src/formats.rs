use std::fmt::{Debug, Display, Formatter};

use windows::Win32::Graphics::Direct3D9::*;

pub struct DisplayFormat(D3DFORMAT);

impl Display for DisplayFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08X}-", self.0.0)
    }
}

pub struct NamedFormat(D3DFORMAT, &'static [(D3DFORMAT, &'static str)]);

impl NamedFormat {
    pub fn new(fmt: D3DFORMAT) -> Self {
        NamedFormat(fmt, &ALL_FORMATS)
    }
}

impl Debug for NamedFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for NamedFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some((_fmt, name)) = self.1.iter().find(|(fmt, _name)| self.0 == *fmt) {
            return write!(f, "{name}");
        }

        DisplayFormat(self.0).fmt(f)
    }
}

const D3DFMT_NV12: D3DFORMAT = D3DFORMAT(u32::from_le_bytes(*b"NV12"));
const D3DFMT_P010: D3DFORMAT = D3DFORMAT(u32::from_le_bytes(*b"P010"));

macro_rules! format_list {
    ($name:ident, $len:expr, [$($codec:expr,)*]) => {
        pub(crate) const $name: [(D3DFORMAT, &'static str); $len] = [
            $(($codec, stringify!($codec))),*
        ];
    }
}

format_list!(
    ALL_FORMATS,
    67,
    [
        D3DFMT_NV12,
        D3DFMT_P010,
        D3DFMT_A1,
        D3DFMT_A1R5G5B5,
        D3DFMT_A2B10G10R10,
        D3DFMT_A2B10G10R10_XR_BIAS,
        D3DFMT_A2R10G10B10,
        D3DFMT_A2W10V10U10,
        D3DFMT_A4L4,
        D3DFMT_A4R4G4B4,
        D3DFMT_A8,
        D3DFMT_A8B8G8R8,
        D3DFMT_A8L8,
        D3DFMT_A8P8,
        D3DFMT_A8R3G3B2,
        D3DFMT_A8R8G8B8,
        D3DFMT_A16B16G16R16,
        D3DFMT_A16B16G16R16F,
        D3DFMT_A32B32G32R32F,
        D3DFMT_BINARYBUFFER,
        D3DFMT_CxV8U8,
        D3DFMT_D15S1,
        D3DFMT_D16,
        D3DFMT_D16_LOCKABLE,
        D3DFMT_D24FS8,
        D3DFMT_D24S8,
        D3DFMT_D24X4S4,
        D3DFMT_D24X8,
        D3DFMT_D32,
        D3DFMT_D32F_LOCKABLE,
        D3DFMT_D32_LOCKABLE,
        D3DFMT_DXT1,
        D3DFMT_DXT2,
        D3DFMT_DXT3,
        D3DFMT_DXT4,
        D3DFMT_DXT5,
        D3DFMT_G8R8_G8B8,
        D3DFMT_G16R16,
        D3DFMT_G16R16F,
        D3DFMT_G32R32F,
        D3DFMT_INDEX16,
        D3DFMT_INDEX32,
        D3DFMT_L6V5U5,
        D3DFMT_L8,
        D3DFMT_L16,
        D3DFMT_MULTI2_ARGB8,
        D3DFMT_P8,
        D3DFMT_Q8W8V8U8,
        D3DFMT_Q16W16V16U16,
        D3DFMT_R3G3B2,
        D3DFMT_R5G6B5,
        D3DFMT_R8G8B8,
        D3DFMT_R8G8_B8G8,
        D3DFMT_R16F,
        D3DFMT_R32F,
        D3DFMT_S8_LOCKABLE,
        D3DFMT_UNKNOWN,
        D3DFMT_UYVY,
        D3DFMT_V8U8,
        D3DFMT_V16U16,
        D3DFMT_VERTEXDATA,
        D3DFMT_X1R5G5B5,
        D3DFMT_X4R4G4B4,
        D3DFMT_X8B8G8R8,
        D3DFMT_X8L8V8U8,
        D3DFMT_X8R8G8B8,
        D3DFMT_YUY2,
    ]
);
