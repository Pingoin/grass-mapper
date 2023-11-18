use nav_types::{ECEF, WGS84};
// You must import in each files when you wants use `t!` macro.
use rust_i18n::t;

rust_i18n::i18n!("locales", fallback = "en");
use gloo_timers::future::TimeoutFuture;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use utils::get_lang_code;

use gloo_events::EventListener;

use leaflet::{Circle, Control, ControlOptions, LatLng, Map, MapOptions, TileLayer};
use wasm_bindgen::JsCast;
use web_sys::{console, window, HtmlAnchorElement};

use crate::utils::create_stored_signal;
mod mutex_box;
mod position;
mod utils;

use crate::position::{get_global_position, start_web_data};

fn main() {
    let lang = get_lang_code();
    let lang: Vec<&str> = lang.as_str().split("-").collect();
    rust_i18n::set_locale(lang[0]);

    sycamore::render(|| {
        view! {
            App{}
        }
    });
}

#[component]
fn App<G: Html>() -> View<G> {
    start_web_data();
    let longitude = create_signal(0.0f64);
    let mower_width = create_stored_signal(String::from("mower_with"), 0.0f64);
    let latitude = create_signal(0.0f64);
    //let accuracy = create_stored_signal(String::from("accuracy"), 0.0f64);
    let altitude = create_signal(0.0f64);

    let options = MapOptions::default();
    //options.set_max_zoom(25.0);

    let result = view! {
        header{}
        main{
            article(class="triple-column"){
                ValueInput(lable=t!("mower_width"),value=mower_width){"m"}
                ValueOutput(lable=t!("longitude"),value=*longitude){""}
                ValueOutput(lable=t!("latitude"),value=*latitude){""}
                ValueOutput(lable=t!("altitude"),value=*altitude){"m"}
            }
            article{
                div(id="map"){}
            }
        }
        footer{

        }
    };

    spawn_local_scoped(async move {
        let map = Map::new("map", &options).locate();
        add_tile_layer(&map);
        add_control(&map);
        let mut last_pos = ECEF::new(0.0f32, 0.0f32, 0.0f32);
        loop {
            if let Some(pos) = get_global_position() {
                let wgs = WGS84::from(pos);

                longitude.set(wgs.longitude_degrees() as f64);
                latitude.set(wgs.latitude_degrees() as f64);
                altitude.set(wgs.altitude() as f64);

                if pos.distance(&last_pos) > 5.0 {
                   map.set_view(
                        &LatLng::new(
                            wgs.latitude_degrees() as f64,
                            wgs.longitude_degrees() as f64,
                        ),
                        18.0,
                    );
                    add_circle(
                        &map,
                        wgs.latitude_degrees() as f64,
                        wgs.longitude_degrees() as f64,
                        mower_width.get(),
                    );
                    last_pos = pos;
                }
            };
            TimeoutFuture::new(1000).await;
        }
    });

    result
}


#[derive(Props)]
pub struct ValueInputProps<G: Html> {
    children: Children<G>,
    lable: String,
    value: Signal<f64>,
}

#[derive(Props)]
pub struct ValueOutputProps<G: Html> {
    children: Children<G>,
    lable: String,
    value: ReadSignal<f64>,
}

#[component]
fn ValueOutput<G: Html>(props: ValueOutputProps<G>) -> View<G> {
    let children = props.children.call();
    view! {
            span{(props.lable)}
            span{(((props.value.get()*100.0).round() / 100.0))}
            div{(children)}
    }
}

#[component]
fn ValueInput<G: Html>(props: ValueInputProps<G>) -> View<G> {
    let children = props.children.call();
    view! {
            span{(props.lable)}
            input(bind:valueAsNumber=props.value, type="number", min="0", step="0.1",maxlength="4",size="8")
            div{(children)}
    }
}

fn add_control(map: &Map) {
    let mut options = ControlOptions::default();
    options.set_position("topleft");
    let control_button = Control::new(&options);

    // This callback must return a HTML div representing the control button.
    let on_add = |_: &_| {
        let document = window()
            .expect("Unable to get browser window")
            .document()
            .expect("Unable to get browser document");

        let container = document
            .create_element("div")
            .expect("Unable to create div");

        container.set_class_name("leaflet-bar");

        let link = document
            .create_element("a")
            .expect("Unable to create link")
            .dyn_into::<HtmlAnchorElement>()
            .expect("Unable to cast to HtmlAnchorElement");

        link.set_href("#");
        link.set_inner_html("⬤");
        link.set_title("Create a new foobar.");

        let on_click = EventListener::new(&link, "click", |_| {
            console::log_1(&"Control button click.".into());
        });

        on_click.forget();

        container
            .append_child(&link)
            .expect("Unable to add child element");

        container.dyn_into().unwrap()
    };

    control_button.on_add(on_add);
    control_button.add_to(map);
}

fn add_tile_layer(map: &Map) {
    TileLayer::new("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png").add_to(map);
}

fn add_circle(map: &Map, lat: f64, lng: f64, radius: f64) {
    Circle::new_with_radius(&LatLng::new(lat, lng), radius).add_to(map);
}
