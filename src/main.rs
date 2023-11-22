mod mutex_box;
mod position;
mod utils;
mod components;

use crate::components::raw_data::RawValues;
use crate::components::{MenuButtons, ValueInput};
use crate::position::{get_global_position, start_web_data};
use crate::utils::create_stored_signal;
use git_version::git_version;
use gloo_timers::future::TimeoutFuture;
use leaflet::{Circle, LatLng, Map, MapOptions, TileLayer};
use nav_types::{ECEF, WGS84};
use rust_i18n::t;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use utils::get_lang_code;

const GIT_VERSION: &str = git_version!(args = ["--always", "--tags"]);
rust_i18n::i18n!("locales", fallback = "en");

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
    
    let mower_width = create_stored_signal(String::from("mower_with"), 0.0f64);
    
    //let accuracy = create_stored_signal(String::from("accuracy"), 0.0f64);

    let menu_visible = create_signal(false);
    let raw_visable = create_signal(false);
    let options = MapOptions::default();
    
    //options.set_max_zoom(25.0);

    let result = view! {
        main{
            div(class="container"){
                div(id="map"){
                    div(class="leaflet-control-container"){
                        div(class="leaflet-bottom leaflet-left"){
                            div(class="leaflet-control-attribution leaflet-control"){
                                (GIT_VERSION)
                            }
                        }
                        div(class="leaflet-top leaflet-right"){
                            (if !(menu_visible.get()||raw_visable.get()){
                                view! {
                                    MenuButtons(raw_visable=raw_visable,menu_visable=menu_visible){}
                        }
                        } else {
                            view! { }
                        })
                        }
                    }
                }
                (if menu_visible.get() {
                    view! {
                        div(class="overlay"){
                            MenuButtons(raw_visable=raw_visable,menu_visable=menu_visible)
                            br{}
                            div(class="triple-column"){
                        ValueInput(lable=t!("mower_width"),value=mower_width){"m"}
                    }}}
                } else {
                    view! { }
                })
                (if raw_visable.get() {
                    view! {
                        RawValues(raw_visable=raw_visable,menu_visable=menu_visible)
                    }
                } else {
                    view! { }
                })


            }
        }
    };

    spawn_local_scoped(async move {
        let map = Map::new("map", &options).locate();
        add_tile_layer(&map);

        //add_control(&map);
        let mut last_pos = ECEF::new(0.0f32, 0.0f32, 0.0f32);
        loop {
            if let Some(pos) = get_global_position() {
                let wgs = WGS84::from(pos);
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









fn add_tile_layer(map: &Map) {
    TileLayer::new("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png").add_to(map);
}

fn add_circle(map: &Map, lat: f64, lng: f64, radius: f64) {
    Circle::new_with_radius(&LatLng::new(lat, lng), radius).add_to(map);
}
