use std::ffi::c_void;

use amml_msvc::MsvcString;

use super::{FakeConnection, FakeHeap, HeapEvent, Vtable};

// a fake `ClientInstance`. Its getGameConnectionInfo writes a GameConnectionInfo of the game layout, `info_size`
// bytes long, with the host string made on FakeHeap, and notes the host's heap block
#[repr(C)]
pub struct FakeClient {
    pub vtable: *const usize,
    pub player: *mut c_void,
    pub world_active: u8,
    pub region: *mut c_void,
    pub connection: Option<FakeConnection>,
    pub host_block: usize,
    pub info_size: u64,
}

impl FakeClient {
    pub const GET_LOCAL_PLAYER: u64 = 1;
    pub const IS_WORLD_ACTIVE: u64 = 2;
    pub const GET_MOUSE_GRABBED: u64 = 3;
    pub const GET_REGION: u64 = 4;
    pub const GET_GAME_CONNECTION_INFO: u64 = 5;
    // sizeof(GameConnectionInfo) as `GameConnectionInfo.size` derives it on 26.51 and on 26.60 (amml data/)
    pub const INFO_SIZE_26_51: u64 = 0x310;
    pub const INFO_SIZE_26_60: u64 = 0x2F0;
    // mType, mHostIpAddress and mPort as the folded entries derived them, the same on 26.51 and 26.60
    const INFO_TYPE: usize = 0;
    const INFO_HOST: usize = 0x8;
    const INFO_PORT: usize = 0x8C;

    pub fn new(player: *mut c_void, region: *mut c_void) -> Box<Self> {
        let player_fn: extern "C" fn(*mut c_void) -> *mut c_void = Self::get_local_player;
        let active: extern "C" fn(*mut c_void) -> u8 = Self::is_world_active;
        let grabbed: extern "C" fn(*mut c_void) -> u8 = Self::get_mouse_grabbed;
        let region_fn: extern "C" fn(*mut c_void) -> *mut c_void = Self::get_region;
        let info: extern "C" fn(*mut c_void, *mut u8) -> *mut u8 = Self::get_game_connection_info;
        let vtable =
            Vtable::leak(&[player_fn as usize, active as usize, grabbed as usize, region_fn as usize, info as usize]);
        Box::new(Self {
            vtable,
            player,
            world_active: 1,
            region,
            connection: None,
            host_block: 0,
            info_size: Self::INFO_SIZE_26_51,
        })
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self).cast()
    }

    fn this<'a>(this: *mut c_void) -> &'a mut FakeClient {
        // SAFETY: the fake's vtable is only reached through a FakeClient
        unsafe { &mut *this.cast::<FakeClient>() }
    }

    extern "C" fn get_local_player(this: *mut c_void) -> *mut c_void {
        Self::this(this).player
    }

    extern "C" fn is_world_active(this: *mut c_void) -> u8 {
        Self::this(this).world_active
    }

    extern "C" fn get_mouse_grabbed(_: *mut c_void) -> u8 {
        1
    }

    extern "C" fn get_region(this: *mut c_void) -> *mut c_void {
        Self::this(this).region
    }

    // `GameConnectionInfo::~GameConnectionInfo()`: frees the host string and notes the object
    pub extern "C" fn destroy_connection_info(this: *mut u8) {
        FakeHeap::record(HeapEvent::Destroy(this as usize));
        // SAFETY: the copy holds the host string made by the getter on FakeHeap; the destructor runs once
        unsafe { (*this.add(Self::INFO_HOST).cast::<MsvcString>()).release(&FakeHeap::msvc()) };
    }

    // `std::optional<GameConnectionInfo> getGameConnectionInfo() const`, through the hidden return pointer
    extern "C" fn get_game_connection_info(this: *mut c_void, out: *mut u8) -> *mut u8 {
        let client = Self::this(this);
        let size = client.info_size as usize;
        let Some(connection) = client.connection.clone() else {
            // SAFETY: the caller passes room for the value and the engaged flag
            unsafe { *out.add(size) = 0 };
            return out;
        };
        // SAFETY: FakeHeap's functions are a valid allocate and free pair
        let host = unsafe { MsvcString::new_in(&FakeHeap::msvc(), &connection.host) }.unwrap();
        client.host_block = host.as_bytes().as_ptr() as usize;
        // SAFETY: the caller passes room for the value and the engaged flag; the offsets are inside the value
        unsafe {
            out.add(Self::INFO_TYPE).cast::<i16>().write_unaligned(connection.connection_type);
            out.add(Self::INFO_HOST).cast::<MsvcString>().write(host);
            out.add(Self::INFO_PORT).cast::<i32>().write_unaligned(connection.port);
            *out.add(size) = 1;
        }
        out
    }
}
