use std::mem::offset_of;

use amml_msvc::MsvcString;

// the head of the game's Social::GameConnectionInfo, up to and with mPort. The head is the same on 26.51 and
// 26.60; the tail is not (the whole value is 0x310 bytes on 26.51 and 0x2F0 on 26.60), so its size comes from the
// `GameConnectionInfo.size` entry
#[repr(C)]
pub struct GameConnectionInfoLayout {
    pub connection_type: i16, // 0x00 mType (Social::ConnectionType)
    _transport: i32,          // 0x04 mTransportType
    pub host: MsvcString,     // 0x08 mHostIpAddress
    _names: [u8; 0x60],       // 0x28 mUnresolvedUrl, mServerRegion, mExternalServerName
    _service_quality: i32,    // 0x88 mServiceQuality
    pub port: i32,            // 0x8C mPort
}

const _: () = assert!(offset_of!(GameConnectionInfoLayout, host) == 0x08);
const _: () = assert!(offset_of!(GameConnectionInfoLayout, port) == 0x8C);
const _: () = assert!(size_of::<GameConnectionInfoLayout>() == 0x90);
