use crate::{mutex_box::MutexBox, utils::log_to_browser};
use chrono::{Datelike, NaiveDateTime, Timelike, Utc};
use libgeomag::{DateTime, GeodeticLocation, ModelExt, IGRF, WMM};
use nalgebra::{Vector2, Vector3, Rotation3};
use nav_types::{ECEF, WGS84};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{self, closure::Closure, JsCast};
use web_sys::{window, DeviceMotionEvent, DeviceOrientationEvent, Geolocation, Position, Window};
mod position_fusion;
use position_fusion::PositionFusion;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

static POSITION_FUSION: MutexBox<PositionFusion> = MutexBox::new_inited(PositionFusion::new());
static RAW_VALUES: MutexBox<RawValues> = MutexBox::new_inited(RawValues::new());

pub fn start_web_data() {
    if let Some(win) = window() {
        if let Ok(geoloc) = win.navigator().geolocation() {
            get_geoloc(&geoloc);
        }
        get_acceleation(&win);
        get_device_orientation(&win);
    }
}

fn get_acceleation(win: &Window) {
    let cb: Closure<dyn Fn(DeviceMotionEvent)> = Closure::new(move |data: DeviceMotionEvent| {
        if let Some(acc) = data.acceleration_including_gravity() {
            //log_to_browser("ACC-Data".to_string());
            if let (Some(x), Some(y), Some(z)) = (acc.x(), acc.y(), acc.z()) {
                let acc_vec = Vector3::new(x as f32, y as f32, z as f32);

                RAW_VALUES.open_locked(
                    |raw| {
                        raw.acceleration = acc_vec;
                        POSITION_FUSION.open_locked(
                            |pos| {
                                pos.predict(raw.acceleration, raw.orientation)
                            },
                            (),
                        );
                    },
                    (),
                );

                //log_to_browser(format!("acc: {}/{}/{}", x, y, z));
            }
        }
    });
    let _bla = win.add_event_listener_with_callback("devicemotion", cb.as_ref().unchecked_ref());
    cb.forget();
}

fn get_geoloc(geoloc: &Geolocation) {
    let cb: Closure<dyn Fn(Position)> = Closure::new(move |data: Position| {
        let coords = data.coords();
        let speed = coords.speed();
        let heading = coords.heading();
        let wgs = WGS84::from_degrees_and_meters(
            coords.latitude() as f32,
            coords.longitude() as f32,
            if let Some(alt) = coords.altitude() {
                alt
            } else {
                0.0
            } as f32,
        );

        let magnetic_declination = calc_magnetic_declination(wgs, Utc::now().naive_utc());
        //log_to_browser("Pos-Data".to_string());
        let coords = ECEF::from(wgs);
        let mut velocity: Option<Vector2<f32>> = None;
        if let (Some(speed), Some(heading)) = (speed, heading) {
            let speed_n = speed * heading.to_radians().cos();
            let speed_e = speed * heading.to_radians().sin();
            //log_to_browser(format!("E/N-Vel: {}/{}", speed_e, speed_n));
            velocity = Some(Vector2::new(speed_e as f32, speed_n as f32));
        }
        //alert(format!("long: {}\nlat: {}\nspeed: {:?}\nheading: {:?}",wgs.longitude_degrees(),wgs.latitude_degrees(),speed,heading).as_str());
        RAW_VALUES.open_locked(
            |raw| {
                raw.position = Some(coords.clone());
                raw.magnetic_declination = magnetic_declination;

                if let Some(vel) = velocity {
                    raw.velocity = vel.clone();
                }
            },
            (),
        );

        POSITION_FUSION.open_locked(
            |pos| {
                pos.update_global_position(coords, velocity);
            },
            (),
        );
    });
    if let Ok(_pos) = geoloc.watch_position(cb.as_ref().unchecked_ref()) {}
    cb.forget();
}

fn get_device_orientation(win: &Window) {
    let cb: Closure<dyn Fn(DeviceOrientationEvent)> =
        Closure::new(move |data: DeviceOrientationEvent| {
            if let (Some(x), Some(y), Some(z)) = (data.alpha(), data.beta(), data.gamma()) {
                let mut orientation_vec = Vector3::new(x.to_radians() as f32 , y.to_radians() as f32, z.to_radians() as f32);

                RAW_VALUES.open_locked(
                    |raw| {
                        orientation_vec[0]=orientation_vec[0]-raw.magnetic_declination;
                        raw.orientation = orientation_vec;
                        POSITION_FUSION.open_locked(
                            |pos| {
                                pos.predict(raw.acceleration, raw.orientation)
                            },
                            (),
                        );
                    },
                    (),
                );
                //log_to_browser(format!("ori: {}/{}/{}", x, y, z));
            }
        });
    let _bla =
        win.add_event_listener_with_callback("deviceorientation", cb.as_ref().unchecked_ref());
    cb.forget();
}

pub fn get_global_position() -> Option<ECEF<f32>> {
    POSITION_FUSION.open_locked(|pos| pos.get_global_position().clone(), None)
}
#[allow(dead_code)]
pub fn reset() {
    POSITION_FUSION.open_locked(|pos| pos.reset(), ())
}

pub fn calc_magnetic_declination(pos: WGS84<f32>, time: NaiveDateTime) -> f32 {
    let l = GeodeticLocation::new(
        pos.longitude_degrees() as f64,
        pos.latitude_degrees() as f64,
        pos.altitude() as f64 / 1000.0,
    );
    let t = DateTime::new(
        time.year() as i32,
        time.month() as i32,
        time.day() as i32,
        time.hour() as i32,
        time.minute() as i32,
        time.second() as i32,
    );

    let wmm = WMM::new(t.decimal).unwrap();
    let igrf = IGRF::new(t.decimal).unwrap();

    let m1 = wmm.single(l).d.to_degrees();
    let m2 = igrf.single(l).d.to_degrees();

    ((m1 + m2) / 2.0).to_radians() as f32
}

pub fn get_raw_data() -> RawValues {
    RAW_VALUES.open_locked(|raw| raw.clone(), RawValues::new())
}

#[derive(Clone, Copy, Debug)]
pub struct RawValues {
    pub position: Option<ECEF<f32>>,
    pub velocity: Vector2<f32>,
    pub orientation: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub magnetic_declination: f32,
}

impl RawValues {
    pub const fn new() -> Self {
        RawValues {
            position: None,
            velocity: Vector2::new(0.0, 0.0),
            orientation: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            magnetic_declination: 0.0,
        }
    }

    pub fn get_acceleration(self)->Vector3<f32> {
        let alpha=self.orientation[0];
        let beta=self.orientation[1];
        let gamma=self.orientation[2];

        let declination= self.magnetic_declination;
        let a_device=self.acceleration;

        // Define the rotation matrices for each axis
        let r_z = Rotation3::from_euler_angles(0.0, 0.0, alpha);

        // Subtract the declination from alpha
        let r_x = Rotation3::from_euler_angles(beta, 0.0, 0.0);
        let r_y = Rotation3::from_euler_angles(0.0, gamma, 0.0);

        // Compute the rotation matrix that transforms device coordinates to world coordinates
        let r = r_z * r_x * r_y;

        // Compute the acceleration vector in world coordinates
        let a_world = r * a_device;

        // Extract the north-south and east-west components of the acceleration vector

        a_world
    }
}
