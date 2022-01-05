use errno::{Errno, errno};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn create_Canota() {
        let canota = super::Canota::new("can0");
        match canota {
            Ok(canota) => println!("Success"),
            Err(e) => panic!(format!("Failed to Create canota struct with error: {:?}", e))
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InitializationError(Errno),
    InvalidStringError(std::ffi::NulError),
    DeviceUnavailable { short_device_id: u8 },
    RawCommandWriteError,
    RawCommandReadError,
    ErrorChangingDeviceShortId
}


pub struct Canota {
    ctx: *mut canota_sys::canota_ctx,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceMode {
    Application,
    Bootloader,
    Unknown
}

// TODO Maybe represent this as a state type instead?
#[derive(Debug, PartialEq, Clone)]
pub enum DeviceType {
    STM32L432KC,
    Unknown
}

impl From<u8> for DeviceType {
    fn from(device_type: u8) -> DeviceType {
        match device_type {
            0 => DeviceType::STM32L432KC,
            _ => DeviceType::Unknown,
        }
    }
}

// TODO IMPL Eq and Ord
#[derive(Clone, Debug)]
pub struct DeviceVersion {
    major: u8,
    minor: u8
}

impl From<u32> for DeviceMode {
    fn from(mode: u32) -> DeviceMode {
        match mode {
            0 => DeviceMode::Application,
            1 => DeviceMode::Bootloader,
            _ => DeviceMode::Unknown
        }
    }
}

#[derive(Clone, Debug)] // Change Default debug to the print_info string format in canota/main.c
pub struct CanotaDeviceInfo {
    short_device_id: u8,
    device_id: u32,
    mode: DeviceMode,
    device_type: DeviceType,
    device_version: DeviceVersion
}

impl From<&canota_sys::canota_device_ctx> for CanotaDeviceInfo {
    fn from(device_ctx: &canota_sys::canota_device_ctx) -> CanotaDeviceInfo {
        CanotaDeviceInfo {
            short_device_id: device_ctx.short_device_id,
            device_id: device_ctx.device_id,
            mode: DeviceMode::from(device_ctx.mode),
            device_type: DeviceType::from(device_ctx.info.device_type),
            device_version: DeviceVersion {
                major: device_ctx.info.version_major,
                minor: device_ctx.info.version_minor
            }
        }
    }
}

/** Valid for Device Identification Responses.
 *  Need to verify for others.
 * May need to change this to be dynamic based on what "type" of can_frame is returned
 */
use byteorder::{ LittleEndian, ByteOrder };
impl From<&canota_sys::can_frame> for CanotaDeviceInfo {
    fn from(frame: &canota_sys::can_frame) -> CanotaDeviceInfo {
        let info = canota_sys::canota_device_ctx__bindgen_ty_1::from(&frame.data[0..3]);
        let device_id = LittleEndian::read_u32(&frame.data[4..7]); // TODO Test this
        CanotaDeviceInfo {
            short_device_id: (frame.can_id & 0xFF) as u8,
            device_id,
            mode: DeviceMode::from(((frame.can_id >> 8) & 0xFF) as u32),
            device_type: DeviceType::from(info.device_type),
            device_version: DeviceVersion {
                major: info.version_major,
                minor: info.version_minor
            }
        }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(error: std::ffi::NulError) -> Self {
        Error::InvalidStringError(error)
    }
}

/**
 * Wraps a canota_device_ctx which only has values that are required for
 * calling canota_raw_cmd. All other values are default and should
 * not be accessed.
 *
 * Valid Values:
 *  - ctx
 * Partially Valid Values
 *  - short_device_id - Only valid if DeviceIdStatus is Valid
 * Values Not Needed and therefor set to meaningless defaults:
 *  - device_id
 *  - mode
 *  - info
 *
 * !INTERNAL: This Should Not be exposed to the end user
 */
struct DeviceIdValid;
struct DeviceIdInvalid;
struct CanotaDeviceCtxRawCmdCompatible<DeviceIdStatus = DeviceIdInvalid> {
    device_ctx: canota_sys::canota_device_ctx,
    device_id_status: std::marker::PhantomData<DeviceIdStatus>
}


impl CanotaDeviceCtxRawCmdCompatible<DeviceIdInvalid>{
    fn new(ctx: *mut canota_sys::canota_ctx) -> CanotaDeviceCtxRawCmdCompatible<DeviceIdInvalid> {
        CanotaDeviceCtxRawCmdCompatible {
            device_ctx: canota_sys::canota_device_ctx {
                ctx,
                device_id: 0,
                short_device_id: 0,
                mode: canota_sys::canota_mode_CANOTA_MODE_APPLICATION,
                info: canota_sys::canota_device_ctx__bindgen_ty_1 {
                    device_type: 0,
                    version_minor: 0,
                    version_major: 0,
                    reserved0: 0
                }
            },
            device_id_status: std::marker::PhantomData
        }
    }

    fn set_short_device_id(self, short_device_id: u8) -> CanotaDeviceCtxRawCmdCompatible<DeviceIdValid> {
        CanotaDeviceCtxRawCmdCompatible {
            device_ctx: canota_sys::canota_device_ctx {
                ctx: self.device_ctx.ctx,
                device_id: 0,
                short_device_id,
                mode: canota_sys::canota_mode_CANOTA_MODE_APPLICATION,
                info: canota_sys::canota_device_ctx__bindgen_ty_1 {
                    device_type: 0,
                    version_minor: 0,
                    version_major: 0,
                    reserved0: 0
                }
            },
            device_id_status: std::marker::PhantomData
        }
    }
}

impl CanotaDeviceCtxRawCmdCompatible<DeviceIdValid> {
    pub fn update_short_device_id(&mut self, short_device_id: u8) {
        self.device_ctx.short_device_id = short_device_id;
    }
}

trait CanotaDeviceIdentificationRequest {
    fn device_identification_request(&mut self) -> Result<(), Error>;
}

impl CanotaDeviceIdentificationRequest for CanotaDeviceCtxRawCmdCompatible<DeviceIdValid> {
    fn device_identification_request(&mut self) -> Result<(), Error> {
        if unsafe { canota_sys::canota_cmd_device_ident_req(&mut self.device_ctx)} {
            Ok(())
        } else {
            Err(Error::RawCommandWriteError)
        }
    }
}


impl Canota {
    pub fn new(can_interface: impl Into<Vec<u8>>) -> Result<Canota, Error> {
        let can_interface = std::ffi::CString::new(can_interface)?;
        let ctx = unsafe { canota_sys::canota_init(can_interface.as_ptr()) }; // AHHHHHH LEAKS memory
        if ctx.is_null() {
            return Err(Error::InitializationError(errno()));
        }
        Ok(Canota {
            ctx
        })
    }

    // TODO Verify if this changes the context object
    pub fn info(&mut self, short_device_id: u8) -> Result<CanotaDeviceInfo, Error> {
        let device = self.get_device_ref(short_device_id)?;
        Ok(CanotaDeviceInfo::from(device))
    }

    fn get_device_ref<'a>(&'a mut self, short_device_id: u8) -> Result<&'a mut canota_sys::canota_device_ctx, Error> {
        // get a device with self and short_device_id
        let device = unsafe { canota_sys::canota_device_from_short_id(self.ctx, short_device_id) }; // AHHHHH Leaks memory
        if device.is_null() {
            Err(Error::DeviceUnavailable { short_device_id })
        } else {
            Ok(unsafe { device.as_ref().unwrap() }) // Check this syntax
        }

    }

    /**
     * Library Uses Poll to read with timeout Internally
     */
    fn raw_receive_frame(&mut self, mask: &mut canota_sys::mask_match, num_matches: u64) -> Result<canota_sys::can_frame, Error> {
        let mut frame = canota_sys::can_frame::new();
        let result = unsafe { canota_sys::canota_raw_recv_frame(self.ctx, &mut frame, mask, num_matches) };
        if result {
           Ok(frame)
        } else {
            Err(Error::RawCommandReadError)
        }
    }

    pub fn scan(&mut self) -> Result<Vec<CanotaDeviceInfo>, Error> {
        let mut device = CanotaDeviceCtxRawCmdCompatible::new(self.ctx).set_short_device_id(0);
        for i in 0..=255 {
            device.update_short_device_id(i);
            device.device_identification_request()?;
        }
        let mut mask_match = canota_sys::mask_match {
            mask:   0x1FFF0000,
            match_: 0x1F010000
        };
        let mut devices = Vec::with_capacity(255); // Most Number of devices that we can have
        while let Ok(frame) = self.raw_receive_frame(&mut mask_match, 1) {
            devices.push(CanotaDeviceInfo::from(&frame))
        }
        Ok(devices)
    }

    pub fn change_short_id(&mut self, old_short_id: u8, new_short_id: u8) -> Result<(), Error> {
        let device = self.get_device_ref(old_short_id)?;
        let result = unsafe { canota_sys::canota_change_short_id(device, new_short_id) };
        if result {
            Ok(())
        } else {
            Err(Error::ErrorChangingDeviceShortId)
        }
    }

    pub fn set_mode(&mut self) { todo!(); }

    pub fn reset_device(&mut self) { todo!(); }

    pub fn erase_section(&mut self) { todo!(); }

    pub fn get_checksum_for_section(&mut self) { todo!(); }

    pub fn flash_board(&mut self) { todo!(); }
}
