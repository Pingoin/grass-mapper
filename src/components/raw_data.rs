use gloo_timers::future::TimeoutFuture;
use nav_types::WGS84;
use rust_i18n::t;
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::{
    components::{MenuButtons, ValueOutput},
    position::get_raw_data, utils::log_to_browser,
};

#[component(inline_props)]
pub fn RawValues<G: Html>(raw_visable: Signal<bool>, menu_visable: Signal<bool>) -> View<G> {
    let altitude = create_signal(0.0f64);
    let magnetic_declination = create_signal(0.0f64);
    let latitude = create_signal(0.0f64);
    let longitude = create_signal(0.0f64);
    let speed_e = create_signal(0.0f64);
    let speed_n = create_signal(0.0f64);

    let acc_e = create_signal(0.0f64);
    let acc_n = create_signal(0.0f64);
    let acc_a = create_signal(0.0f64);

    spawn_local_scoped(async move {
        loop {
            let data = get_raw_data();

            if let Some(pos) = data.position {
                let wgs = WGS84::from(pos);
                latitude.set(wgs.latitude_degrees() as f64);
                longitude.set(wgs.longitude_degrees() as f64);
                altitude.set(wgs.altitude() as f64);
            }
            magnetic_declination.set(data.magnetic_declination.to_degrees() as f64);

            speed_e.set(data.velocity[0] as f64);
            speed_n.set(data.velocity[1] as f64);

            let acc=data.get_acceleration();
            acc_a.set(acc[2] as f64);
            acc_n.set(acc[0] as f64);
            acc_e.set(acc[1] as f64);
            log_to_browser(format!("Raw-Vals: {:?}",data));
            TimeoutFuture::new(1000).await;
        }
    });

    view! {
        div(class="overlay"){
            MenuButtons(raw_visable=raw_visable,menu_visable=menu_visable)
            br{}
            div(class="triple-column"){
        ValueOutput(lable=t!("longitude"),value=*longitude){""}
        ValueOutput(lable=t!("latitude"),value=*latitude){""}
        ValueOutput(lable=t!("altitude"),value=*altitude){"m"}
        ValueOutput(lable=t!("magnetic_declination"),value=*magnetic_declination){(t!("degree"))}
        ValueOutput(lable=t!("speed east"),value=*speed_e){"m/s"}
        ValueOutput(lable=t!("speed_north"),value=*speed_n){"m/s"}
        ValueOutput(lable=t!("acceleration east"),value=*acc_e){"m/s²"}
        ValueOutput(lable=t!("acceleration_north"),value=*acc_n){"m/s²"}
        ValueOutput(lable=t!("acceleration altitude"),value=*acc_a){"m/s²"}

            }
    }}
}
