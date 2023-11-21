use crate::mutex_box::MutexBox;
use chrono::{NaiveDateTime, Datelike, Timelike};
use libgeomag::{DateTime, GeodeticLocation, ModelExt, IGRF, WMM};
use nalgebra::Vector2;
use nav_types::{ECEF, WGS84};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, Position};
mod position_fusion;
use position_fusion::PositionFusion;

static POSITION_FUSION: MutexBox<PositionFusion> = MutexBox::new_inited(PositionFusion::new());

pub fn start_web_data() {
    if let Some(win) = window() {
        if let Ok(geoloc) = win.navigator().geolocation() {
            let cb: Closure<dyn Fn(Position)> = Closure::new(move |data: Position| {
                let coords = data.coords();
                let speed=coords.speed();
                let heading=coords.heading();
                let coords = ECEF::from(WGS84::from_degrees_and_meters(
                    coords.latitude() as f32,
                    coords.longitude() as f32,
                    if let Some(alt) = coords.altitude() {
                        alt
                    } else {
                        0.0
                    } as f32,
                ));
                let mut velocity:Option<Vector2<f32>>=None;
                if let (Some(speed),Some(heading)) = (speed,heading)  {
                    let speed_n=speed*heading.to_radians().cos();
                    let speed_e=speed*heading.to_radians().sin();
                   velocity=Some(Vector2::new(speed_e as f32,speed_n as f32));
                }

                POSITION_FUSION.open_locked(
                    |pos| {
                        pos.update_global_position(coords,velocity);
                    },
                    (),
                );
            });
            if let Ok(_pos) = geoloc.watch_position(cb.as_ref().unchecked_ref()) {}
            cb.forget();
        }
    }
}

pub fn get_global_position() -> Option<ECEF<f32>> {
    POSITION_FUSION.open_locked(|pos| pos.get_global_position().clone(), None)
}
#[allow(dead_code)]
pub fn reset(){
    POSITION_FUSION.open_locked(|pos| pos.reset(), ())
} 

pub fn calc_magnetic_declination(pos:WGS84<f32>,time:NaiveDateTime)->f64{
    let l = GeodeticLocation::new(
        pos.longitude_degrees() as f64,
        pos.latitude_degrees() as f64,
        pos.altitude() as f64 / 1000.0,
    );
    let t = DateTime::new(
        time.year() as i32,
        time.month() as i32,
        time.day() as i32,
        time.hour()as i32,
        time.minute()as i32,
        time.second()as i32,
    );

    let wmm = WMM::new(t.decimal).unwrap();
    let igrf = IGRF::new(t.decimal).unwrap();

    let m1 = wmm.single(l).d.to_degrees();
    let m2 = igrf.single(l).d.to_degrees();

    (m1+m2)/2.0
}