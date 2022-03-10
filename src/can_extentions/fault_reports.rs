pub enum SeverityCode {
    SEVERE,
    DANGER,
    WARNING,
    UNKNOWN
}
impl From<u8> for SeverityCode {
    fn from(other: u8) -> SeverityCode {
        match other {
            0x0 => SeverityCode::SEVERE,
            0x1 => SeverityCode::DANGER,
            0x2 => SeverityCode::WARNING,
            _ => SeverityCode::UNKNOWN,
        }
    }  
}

#[allow(non_camel_case_types)]
pub enum BmsErrorCode {
    BATTERY_OVERVOLTAGE,
    BATTERY_UNDERVOLTAGE,
    BATTERY_OVERCURRENT,
    BATTERY_SOC,
    CELL_UNDERVOLTAGE,
    CELL_OVERVOLTAGE,
    CELL_TEMPERATURE,
    BUCK_TEMPERATURE,
    LOW_LAYER_EXCEPTION,
    UNKNOWN
}
impl From<u8> for BmsErrorCode {
    fn from(other: u8) -> BmsErrorCode {
        match other {
            0x0 => BmsErrorCode::BATTERY_OVERVOLTAGE,
            0x1 => BmsErrorCode::BATTERY_UNDERVOLTAGE,
            0x2 => BmsErrorCode::BATTERY_OVERCURRENT,
            0x3 => BmsErrorCode::BATTERY_SOC,
            0x4 => BmsErrorCode::CELL_UNDERVOLTAGE,
            0x5 => BmsErrorCode::CELL_OVERVOLTAGE,
            0x6 => BmsErrorCode::CELL_TEMPERATURE,
            0x7 => BmsErrorCode::BUCK_TEMPERATURE,
            0x8 => BmsErrorCode::LOW_LAYER_EXCEPTION,
            _ => BmsErrorCode::UNKNOWN
        }
    }
}

impl From<&[u8]> for BmsFaultReport {
    fn from(other: &[u8]) -> BmsFaultReport {
        BmsFaultReport {
            severity_code: SeverityCode::from(other[0]),
            error_code: BmsErrorCode::from(other[1])
        }
    }
}
pub struct BmsFaultReport {
    pub severity_code: SeverityCode,
    pub error_code: BmsErrorCode,
}

impl From<&[u8]> for MotorControllerFaultReport {
    fn from(other: &[u8]) -> MotorControllerFaultReport {
        MotorControllerFaultReport {
            severity_code: SeverityCode::from(other[0]),
        }
    }
}
pub struct MotorControllerFaultReport {
    pub severity_code: SeverityCode
    // TODO Get the Error code values when they're available
}