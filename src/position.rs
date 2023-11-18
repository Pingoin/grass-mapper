use crate::mutex_box::MutexBox;
use nav_types::{ECEF, WGS84};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, Position};

static POSITIOM_FUSION: MutexBox<PositionFusion> = MutexBox::new_inited(PositionFusion::new());

pub fn start_web_data() {
    if let Some(win) = window() {
        if let Ok(geoloc) = win.navigator().geolocation() {
            let cb: Closure<dyn Fn(Position)> = Closure::new(move |data: Position| {
                let coords = data.coords();
                POSITIOM_FUSION.open_locked(
                    |pos| {
                        pos.update_global_position(ECEF::from(WGS84::from_degrees_and_meters(
                            coords.latitude() as f32,
                            coords.longitude() as f32,
                            if let Some(alt) = coords.altitude() {
                                alt
                            } else {
                                0.0
                            } as f32,
                        )));
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
    POSITIOM_FUSION.open_locked(|pos| pos.reference_position.clone(), None)
}

struct PositionFusion {
    reference_position: Option<ECEF<f32>>,
    tracking_active: bool,
}

impl PositionFusion {
    const fn new() -> Self {
        PositionFusion {
            reference_position: None,
            tracking_active: false,
        }
    }

    fn update_global_position(&mut self, pos: ECEF<f32>) {
        if self.tracking_active {
        } else {
            self.reference_position = Some(pos);
        }
    }
}

impl Default for PositionFusion {
    fn default() -> Self {
        Self::new()
    }
}
