// You must import in each files when you wants use `t!` macro.
use rust_i18n::t;

rust_i18n::i18n!("locales", fallback = "en");
use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use utils::get_lang_code;
use gloo_timers::future::TimeoutFuture;

use crate::utils::create_stored_signal;
mod utils;
mod position;
mod mutex_box;

use crate::position::{start_web_data, get_global_position};

fn main() {
    let lang=get_lang_code();
    let lang: Vec<&str>=lang.as_str().split("-").collect();
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
    let longitude:Signal<f64> = create_stored_signal(String::from("long"),0.0f64);
    let mower_width:Signal<f64> = create_stored_signal(String::from("mower_with"),0.0f64);
    let latitude =
        create_stored_signal(String::from("lat"), 0.0f64);
    //let accuracy = create_stored_signal(String::from("accuracy"), 0.0f64);
    let altitude = create_stored_signal(String::from("altitude"), 0.0f64);



    spawn_local_scoped( async move {
        loop {
            if let Some(pos)=get_global_position(){
                longitude.set(pos.longitude);
                latitude.set(pos.latitude);
                altitude.set(pos.altitude);
            };

            
   
            TimeoutFuture::new(100).await;
        }
    });


    view! {
        header{}
        main{
            article(class="triple-column"){
                ValueInput(lable=t!("mower_width"),value=mower_width){"m"}
                ValueOutput(lable=t!("longitude"),value=*longitude){""}
                ValueOutput(lable=t!("latitude"),value=*latitude){""}
                ValueOutput(lable=t!("altitude"),value=*altitude){"m"}
            }
        }
        footer{

        }
    }
}


#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
struct Currency {
    code: String,
    rate: f64,
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
