use std::fmt::{Debug, Display, Formatter};
use serde::{Serialize, Serializer};
use windows::core::*;
use windows::Win32::Media::DirectShow::*;

pub struct NamedGuid(GUID, &'static [(GUID, &'static str)]);

impl NamedGuid {
    pub fn new(guid: GUID) -> Self {
        NamedGuid(guid, &ALL_GUIDS)
    }
}

impl Debug for NamedGuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for NamedGuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some((_guid, name)) = self.1.iter().find(|(guid, _name)| self.0 == *guid) {
            return write!(f, "{name}");
        }

        Display::fmt(&DisplayGuid(self.0), f)
    }
}

#[derive(Clone)]
pub struct DisplayGuid(pub GUID);

impl Serialize for DisplayGuid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&format!("{self}"))
    }
}

impl Debug for DisplayGuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for DisplayGuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08X}-", self.0.data1)?;
        write!(f, "{:04X}-", self.0.data2)?;
        write!(f, "{:04X}-", self.0.data3)?;
        write!(f, "{:02X}{:02X}-", self.0.data4[0], self.0.data4[1])?;

        for b in &self.0.data4[2..] {
            write!(f, "{:02X}", b)?;
        }

        Ok(())
    }
}

#[allow(non_upper_case_globals)]
const DXVADDI_Intel_ModeH264_E: GUID = GUID::from_values(
    0x604F8E68,
    0x4951,
    0x4c54,
    [0x88, 0xFE, 0xAB, 0xD2, 0x5C, 0x15, 0xB3, 0xD6],
);

macro_rules! codec_list {
    ($name:ident, $len:expr, [$($codec:expr,)*]) => {
        pub(crate) const $name: [(GUID, &'static str); $len] = [
            $(($codec, stringify!($codec))),*
        ];
    }
}

codec_list!(
    ALL_GUIDS,
    49,
    [
        DXVA_ModeAV1_VLD_12bit_Profile2,
        DXVA_ModeAV1_VLD_12bit_Profile2_420,
        DXVA_ModeAV1_VLD_Profile0,
        DXVA_ModeAV1_VLD_Profile1,
        DXVA_ModeAV1_VLD_Profile2,
        DXVA_ModeH261_A,
        DXVA_ModeH261_B,
        DXVA_ModeH263_A,
        DXVA_ModeH263_B,
        DXVA_ModeH263_C,
        DXVA_ModeH263_D,
        DXVA_ModeH263_E,
        DXVA_ModeH263_F,
        DXVA_ModeH264_A,
        DXVA_ModeH264_B,
        DXVA_ModeH264_C,
        DXVA_ModeH264_D,
        DXVA_ModeH264_E,
        DXVA_ModeH264_F,
        DXVA_ModeH264_VLD_Multiview_NoFGT,
        DXVA_ModeH264_VLD_Stereo_NoFGT,
        DXVA_ModeH264_VLD_Stereo_Progressive_NoFGT,
        DXVA_ModeH264_VLD_WithFMOASO_NoFGT,
        DXVA_ModeHEVC_VLD_Main,
        DXVA_ModeHEVC_VLD_Main10,
        DXVA_ModeMPEG1_A,
        DXVA_ModeMPEG1_VLD,
        DXVA_ModeMPEG2_A,
        DXVA_ModeMPEG2_B,
        DXVA_ModeMPEG2_C,
        DXVA_ModeMPEG2_D,
        DXVA_ModeMPEG2and1_VLD,
        DXVA_ModeMPEG4pt2_VLD_AdvSimple_GMC,
        DXVA_ModeMPEG4pt2_VLD_AdvSimple_NoGMC,
        DXVA_ModeMPEG4pt2_VLD_Simple,
        DXVA_ModeNone,
        DXVA_ModeVC1_A,
        DXVA_ModeVC1_B,
        DXVA_ModeVC1_C,
        DXVA_ModeVC1_D,
        DXVA_ModeVC1_D2010,
        DXVA_ModeVP8_VLD,
        DXVA_ModeVP9_VLD_10bit_Profile2,
        DXVA_ModeVP9_VLD_Profile0,
        DXVA_ModeWMV8_A,
        DXVA_ModeWMV8_B,
        DXVA_ModeWMV9_A,
        DXVA_ModeWMV9_B,
        DXVA_ModeWMV9_C,
    ]
);
