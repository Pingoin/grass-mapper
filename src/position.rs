use web_sys::{window, Position};
use wasm_bindgen::{closure::Closure, JsCast};
use crate::mutex_box::MutexBox;

static POSITIOM_FUSION:MutexBox<PositionFusion>=MutexBox::new_inited(PositionFusion::new());


pub fn start_web_data(){
    if let Some(win) = window() {
        if let Ok(geoloc) =win.navigator().geolocation()  {
            let cb: Closure<dyn Fn(Position)>  = Closure::new(move |data:Position| { 
                let coords=data.coords();
                POSITIOM_FUSION.open_locked(|pos|{
                    pos.latitude=coords.latitude();
                    pos.longitude=coords.longitude();
                
                    pos.altitude= if let Some(alt) = coords.altitude() {
                        alt
                    }else{
                        0.0
                    };
                }, ());
            });
            //let cb = cb.as_ref().unchecked_ref();
            if let Ok(_pos) = geoloc.watch_position(cb.as_ref().unchecked_ref()) {
                
            }
            cb.forget();
        }
    }
}

pub fn get_global_position()->Option<GlobalPosition>{

    POSITIOM_FUSION.open_locked(|pos|{
        Some(
            GlobalPosition { longitude: pos.longitude, latitude: pos.latitude, altitude: pos.longitude }
        )
    }, None)
}

pub struct GlobalPosition{
    pub longitude:f64,
    pub latitude:f64,
    pub altitude:f64,
}



struct PositionFusion{
    longitude:f64,
    latitude:f64,
    altitude:f64,

}

impl PositionFusion {
    pub const fn new()->Self{
        PositionFusion { longitude: 0.0, latitude: 0.0, altitude: 0.0 }
    }
}

impl Default for PositionFusion {
     fn default() -> Self {
        Self { longitude: Default::default(), latitude: Default::default(), altitude: Default::default() }
    }
}