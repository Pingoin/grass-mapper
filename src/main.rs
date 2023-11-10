use std::collections::HashMap;

// You must import in each files when you wants use `t!` macro.
use rust_i18n::t;

rust_i18n::i18n!("locales",fallback="en");
use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use utils::get_lang_code;

use crate::utils::{create_stored_signal, fetch};
mod utils;

fn main() {
    rust_i18n::set_locale(get_lang_code().as_str());

    sycamore::render(|| {
        view! {
            App{}
        }
    });
}

#[component]
fn App<G: Html>() -> View<G> {
    let currency: Signal<HashMap<String, Currency>> =
        create_stored_signal(String::from("currencies"), HashMap::new());

    let price_nearby = create_stored_signal(String::from("price_nearby"), 1.779f64);

    let currency_nearby =
        create_stored_signal(String::from("currency_nearby"), String::from("eur"));
    let price_far = create_stored_signal(String::from("price_far"), 5.779f64);
    let currency_far = create_stored_signal(String::from("currency_far"), String::from("pln"));

    let fuel_usage = create_stored_signal(String::from("fuel_usage"), 5.0f64);
    let fueling_detour_km = create_stored_signal(String::from("fueling_detour_km"), 50.0f64);
    let fuel_amount = create_stored_signal(String::from("fuel amount"), 45.0f64);

    let conversion_factor = create_memo(move || {
        let near_string = currency_nearby.with(|cur| cur.clone());
        let far_string = currency_far.with(|cur| cur.clone());

        let near = currency.with(|cur| {
            if let Some(val) = cur.get_key_value(&near_string) {
                val.1.rate
            } else {
                1.0
            }
        });

        let far = currency.with(|cur| {
            if let Some(val) = cur.get_key_value(&far_string) {
                val.1.rate
            } else {
                1.0
            }
        });
        near / far
    });

    let price_far_converted = create_memo(move || price_far.get() * conversion_factor.get());

    let fuel_kosts_near = create_memo(move || price_nearby.get() * fuel_amount.get());

    let fuel_kosts_far = create_memo(move || price_far_converted.get() * fuel_amount.get());

    let detour_kosts = create_memo(move || {
        price_far_converted.get() * fuel_usage.get() * fueling_detour_km.get() / 100.0
    });

    let savings =
        create_memo(move || fuel_kosts_near.get() - fuel_kosts_far.get() - detour_kosts.get());

    currency.track();
    spawn_local_scoped(async move {
        fetch("https://www.floatrates.com/daily/eur.json", |response| {
            if let Ok(devs) = serde_json::from_str::<HashMap<String, Currency>>(&response) {
                currency.set(devs);
            }
        })
        .await;
    });
    view! {
        header{(get_lang_code())}
        main{
            article(class="triple-column"){
                ValueInput(lable=t!("price_nearby"),value=price_nearby){
                    select(bind:value=currency_nearby){
                        CurrencyOptions{}
                    }
                }
                ValueInput(lable=t!("price_far"),value=price_far){
                    select(bind:value=currency_far){
                        CurrencyOptions{}
                    }
                }
                (if conversion_factor.get() !=1.0{
                    view!{
                        ValueOutput(lable=t!("conversion_factor"),value=conversion_factor){""}
                        ValueOutput(lable=t!("price_far_converted"),value=price_far_converted){"€/l"}
                    }

                } else {
                    view! { } // Now you don't
                })
                ValueInput(lable=t!("fuel_usage"),value=fuel_usage){"l/100 km"}
                ValueInput(lable=t!("fueling_detour_km"),value=fueling_detour_km){"km"}
                ValueInput(lable=t!("fuel_amount"),value=fuel_amount){"l"}
                ValueOutput(lable=t!("fuel_costs_near"),value=fuel_kosts_near){"€"}
                ValueOutput(lable=t!("fuel_costs_far"),value=fuel_kosts_far){"€"}
                ValueOutput(lable=t!("detour_kosts"),value=detour_kosts){"€"}
                ValueOutput(lable=t!("savings"),value=savings){"€"}
            }
        }
        footer{
            a(href="https://www.floatrates.com"){
                "Currency values from https://www.floatrates.com"
            }
        }
    }
}

#[component]
fn CurrencyOptions<G: Html>() -> View<G> {
    let supported_currencies = vec![("€", "eur"), ("zł", "pln")];
    View::new_fragment(
        supported_currencies
            .iter()
            .map(|&x| view! { option(value=x.1) { (x.0) } })
            .collect(),
    )
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
