mod components;
mod mutex_box;
mod position;
mod utils;

use crate::components::raw_data::RawValues;
use crate::components::{MenuButtons, ValueInput};
use crate::position::{get_global_position, start_web_data};
use crate::utils::create_stored_signal;
use git_version::git_version;
use gloo_timers::future::TimeoutFuture;
use nav_types::ECEF;
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

    let result = view! {
        header{
            div{}
            div{}
            div{(GIT_VERSION)}
        }
        nav{
            div(class="overlay"){
                MenuButtons(raw_visable=raw_visable,menu_visable=menu_visible)
                br{}
                div(class="triple-column"){
            ValueInput(lable=t!("mower_width"),value=mower_width){"m"}
            RawValues(raw_visable=raw_visable,menu_visable=menu_visible)
        }}
            
        }
        main{
            div(class="container"){
               svg(width="300", height="200", viewbox="0 0 300 200"){
                desc{"FLagge Homoland"}
                rect(x="0",y="0",width="100",height="200", fill="#0055a4")
               }
            }
        }
        footer{}
    };

    spawn_local_scoped(async move {
        let mut last_pos = ECEF::new(0.0f32, 0.0f32, 0.0f32);
        loop {
            if let Some(pos) = get_global_position() {
                if pos.distance(&last_pos) > 5.0 {
                    last_pos = pos;
                }
            };
            TimeoutFuture::new(1000).await;
        }
    });

    result
}
