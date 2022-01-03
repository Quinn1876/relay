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
    InvalidStringError(std::ffi::FromVecWithNulError),
    DeviceUnavailable(u8), // TODO: Make this a named field
    RawCommandWriteError,
    RawCommandReadError,
}


pub struct Canota {
    ctx: *mut canota_sys::canota_ctx,
}

#[derive(Clone, Debug, ParEq)]
pub enum DeviceMode {
    Application,
    Bootloader,
    Unknown
}

// TODO Maybe represent this as a state type instead?
#[derive(Debug, ParEq, Clone)]
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

impl From<u8> for DeviceMode {
    fn from(mode: u8) -> DeviceMode {
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
    version: DeviceVersion
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

impl From<std::ffi::FromVecWithNulError> for Error {
    fn from(error: std::ffi::FromVecWithNulError) -> Self {
        Error::InvalidStringError(error)
    }
}


struct DeviceIdValid {}
struct DeviceIdInvalid {}

trait DeviceIdStatus;
impl DeviceIdStatus for DeviceIdValid;
impl DeviceIdStatus for DeviceIdInvalid;

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
struct CanotaDeviceCtxRawCmdCompatible<T: impl DeviceIdStatus> {
    device_ctx: canota_sys::canota_device_ctx
    device_id_status: std::marker::PhantomData<T>
}

impl<T> CanotaDeviceCtxRawCmdCompatible<T> {
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
            device_id_status: std::marker::PhantomData<DeviceIdInvalid>
        }
    },
}
impl CanotaDeviceCtxRawCmdCompatible<DeviceIdInvalid>{
    fn set_short_device_id(self, short_device_id: u8) -> CanotaDeviceCtxRawCmdCompatible<DeviceIdValid> {
        CanotaDeviceCtxRawCmdCompatible {
            device_ctx: canota_sys::canota_device_ctx {
                ctx: self.ctx,
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
            device_id_status: std::marker::PhantomData<DeviceIdValid>
        }
    }
}

impl CanotaDeviceCtxRawCmdCompatible<DeviceIdValid> {
    pub fn update_short_device_id(&mut self, short_device_id: u8) {
        self.device_ctx.short_device_id = short_device_id;
    }
}

trait CanotaDeviceIdentificationRequest {
    fn device_identification_request(&self) -> Result<(), Error>;
}

impl CanotaDeviceIdentificationRequest for CanotaDeviceCtxRawCmdCompatible<DeviceIdValid> {
    fn device_identification_request(&self) -> Result<(), Error> {
        if unsafe { canota_sys::canota_cmd_device_ident_req(self.device.as_ptr())} {
            Ok(())
        } else {
            Err(Error::RawCommandWriteError)
        }
    }
}


impl Canota {
    pub fn new(can_interface: impl Into<Vec<u8>>) -> Result<Canota, Error> {
        let can_interface = std::ffi::CString::from_vec_with_nul(can_interface.into())?;
        let ctx = unsafe { canota_sys::canota_init(can_interface.as_ptr()) };
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

    fn get_device_ref<'a>(&mut 'a self, short_device_id: u8) -> Result<&'a canota_sys::canota_device_ctx, Error> {
        // get a device with self and short_device_id
        let device = unsafe { canota_sys::canota_device_from_short_id(self.ctx, short_device_id) };
        if device.is_null() {
            Err(Error::DeviceUnavailable(short_device_id))
        }
        Ok(device.as_ref()) // Check this syntax

    }

    /**
     * Library Uses Poll to read with timeout Internally
     */
    fn raw_receive_frame(&mut self, mask: &canota_sys::mask_match, num_matches: usize) -> Result<canota_sys::can_frame, Error> {
        let frame = canota_sys::can_frame;
        let result = unsafe { canota_sys::canota_raw_recv_frame(self.ctx, frame.as_ptr_mut(), mask.as_ptr(), num_matches) };
        if result {
           Ok(frame)
        } else {
            Err(Error::RawCommandReadError)
        }
    }

    pub fn scan(&mut self) -> Result<(), Error> {
        let device = CanotaDeviceCtxRawCmdCompatible::new(self.ctx).set_short_device_id(0);
        for i in 0..=255 {
            device.set_short_device_id(i);
            device.device_identification_request()?;
        }
        let mask_match = canota_sys::mask_match {
            mask:   0x1FFF0000,
            match_: 0x1F010000
        };
        while Ok(())
    }
}
