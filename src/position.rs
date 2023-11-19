use crate::mutex_box::MutexBox;
use nav_types::{ECEF, WGS84};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, Position};
mod position_fusion;
use position_fusion::PositionFusion;

static POSITIOM_FUSION: MutexBox<PositionFusion> = MutexBox::new_inited(PositionFusion::new());

pub fn start_web_data() {
    if let Some(win) = window() {
        if let Ok(geoloc) = win.navigator().geolocation() {
            let cb: Closure<dyn Fn(Position)> = Closure::new(move |data: Position| {
                let coords = data.coords();
                let coords = ECEF::from(WGS84::from_degrees_and_meters(
                    coords.latitude() as f32,
                    coords.longitude() as f32,
                    if let Some(alt) = coords.altitude() {
                        alt
                    } else {
                        0.0
                    } as f32,
                ));
                POSITIOM_FUSION.open_locked(
                    |pos| {
                        pos.update_global_position(coords);
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
    POSITIOM_FUSION.open_locked(|pos| pos.get_global_position().clone(), None)
}

