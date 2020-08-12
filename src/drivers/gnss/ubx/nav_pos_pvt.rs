use crate::datastructures::coordinate;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Valid(u8);

impl Valid {
    pub fn valid_date(self) -> bool {
        self.0 & (1 << 0) > 0
    }

    pub fn valid_time(self) -> bool {
        self.0 & (1 << 1) > 0
    }

    pub fn full_resolved(self) -> bool {
        self.0 & (1 << 2) > 0
    }

    pub fn valid_magnetic(self) -> bool {
        self.0 & (1 << 2) > 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum FixType {
    NoFix = 0,
    DeadReckoningOnly,
    TwoDemension,
    ThreeDemension,
    GNSSPlusDeadReckoningCombined,
    TimeOnlyFix,
}

impl Default for FixType {
    fn default() -> Self {
        Self::NoFix
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Flags1(u8);

impl Flags1 {
    pub fn gnss_fix_ok(self) -> bool {
        self.0 & (1 << 0) > 0
    }

    pub fn heading_of_vehicle_valid(self) -> bool {
        self.0 & (1 << 5) > 0
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Flags2(u8);

impl Flags2 {
    pub fn confirmed_available(self) -> bool {
        self.0 & (1 << 5) > 0
    }

    pub fn confirmed_date(self) -> bool {
        self.0 & (1 << 6) > 0
    }

    pub fn confirmed_time(self) -> bool {
        self.0 & (1 << 7) > 0
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Flags3(u8);

impl Flags3 {
    pub fn invalid_lon_lat_height_msl(self) -> bool {
        self.0 & (1 << 0) > 0
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Longitude(i32);

impl Into<coordinate::Longitude> for Longitude {
    fn into(self) -> coordinate::Longitude {
        coordinate::Longitude(((self.0 as i64 * 3600) * 128 / 1000_000) as i32)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Latitude(i32);

impl Into<coordinate::Latitude> for Latitude {
    fn into(self) -> coordinate::Latitude {
        coordinate::Latitude(((self.0 as i64 * 3600) * 128 / 1000_000) as i32)
    }
}

#[derive(Debug, Default, PartialEq)]
#[repr(C)]
pub struct NavPositionVelocityTime {
    pub itow: u32,

    pub year: u16,
    pub month: u8,
    pub day: u8,

    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub valid: Valid,

    pub time_accuracy_estimate: u32, // ns
    pub nano: i32,                   // -1e9 .. 1e9

    pub fix_type: FixType,
    pub flags1: Flags1,
    pub flags2: Flags2,
    pub num_satellites: u8,

    pub longitude: Longitude,     // 1e-7 degree
    pub latitude: Latitude,       // 1e-7 degree
    pub height: i32,              // height above ellipsoid, unit mm
    pub height_above_msl: i32,    // unit mm
    pub horizental_accuracy: u32, // unit mm
    pub vertical_accuracy: u32,   // unit mm
    pub velocity_north: i32,      // unit mm/s
    pub velocity_east: i32,       // unit mm/s
    pub velocity_down: i32,       // unit mm/s
    pub ground_speed: i32,        // unit mm/s
    pub heading_of_motion: i32,   // 1e-5 unit degree
    pub speed_accuracy: u32,      // unit mm/s
    pub heading_accuracy: u32,    // 1e-5 unit degree

    pub position_dop: u16, // unit 0.01
    pub flags3: Flags3,
    pub _reserved: [u8; 5],
    pub heading_of_vehicle: i32,   // 1e-5 degree
    pub magnetic_declination: i16, // 1e-2 degree
    pub magnetic_accuracy: u16,    // 1e-2 degree
}

mod test {
    #[test]
    fn test_ubx_nav_pos_pvt() {
        use crate::datastructures::coordinate::{Latitude, Longitude};

        use super::NavPositionVelocityTime;

        assert_eq!(core::mem::size_of::<NavPositionVelocityTime>(), 92);

        let message = hex!(
            "00 00 00 00 E0 07 0A 15 16 0D 0A 04 01 00 00 00
             01 00 00 00 03 0C E0 0B 86 BE 2F FF AD 1F 21 20
             E0 F2 09 00 A0 56 09 00 01 00 00 00 01 00 00 00
             00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
             00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
             00 00 00 00 00 00 00 00 00 00 00 00"
        );
        let nav_pos_pvt: &NavPositionVelocityTime =
            unsafe { core::mem::transmute(message.as_ptr()) };
        assert_eq!(nav_pos_pvt.year, 2016);
        let longitude: Longitude = nav_pos_pvt.longitude.into();
        assert_eq!("W001°21.533", format!("{}", longitude));
        let latitude: Latitude = nav_pos_pvt.latitude.into();
        assert_eq!("N53°54.150", format!("{}", latitude));
    }
}
