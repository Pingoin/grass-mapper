use gloo_net::http::Request;
use sycamore::{prelude::*, web::html::nav};
use web_sys::{window};
use serde::{ Deserialize, Serialize};

pub async fn fetch<Fn>(path: &str, fun: Fn)
where
    Fn: FnOnce(String),
{
    if let Ok(response) = Request::get(path).send().await {
        if !response.ok() {
    
        } else {
            let response = response.text().await.expect("Decode");
            fun(response)
        }
    }
}

#[allow(dead_code)]
pub async fn post<Fn, T>(path: &str, data: T, fun: Fn) 
where
    Fn: FnOnce(String) ,
    T: serde::Serialize,
{
    if let Ok(request) = Request::post(path).json(&data) {
        if let Ok(response) = request.send().await {
            if !response.ok() {
             
            } else {
                let response = response.text().await.expect("Decode");
                fun(response)
            }
        }
    } 
}

pub fn get_lang_code()->String{
    let mut result="en".to_string();
    if let Some(win) = window() {
        let navi=win.navigator();
        if let Some(res) = navi.language()  {
         result=res;   
        }
    }


    result
}

pub fn get_stored_text(key: &str, default: String) -> String {
    let mut result = default;
    if let Some(win) = window() {
        if let Ok(Some(stor)) = win.local_storage() {
            if let Ok(Some(store_result)) = stor.get(key).clone() {
                result = store_result;
            }
        }
    }
    result
}

pub fn set_stored_text(key: &str, value: String) {
    if let Some(win) = window() {
        if let Ok(Some(stor)) = win.local_storage() {
            let _ = stor.set(key, &value);
        }
    }
}

pub fn get_stored_item<T>(key: &str, default_value: T) -> T
where
    T: for<'a> Deserialize<'a>,
{
    let mut result = default_value;

    let text = get_stored_text(key, String::new());
    {
        if let Ok(data) = serde_json::from_str(&text) {
            result = data;
        }
    }
    result
}

pub fn set_stored_item<T>(key: &str, value: T)
where
    T: Serialize,
{
    if let Ok(string) = serde_json::to_string_pretty(&value) {
        set_stored_text(key, string);
    }
}

pub fn create_stored_signal<T>(key: String, default_value: T) -> Signal<T>
where
    T: for<'a> Deserialize<'a> + Serialize + Clone,
{
    let result = create_signal(get_stored_item(key.as_str(), default_value));
    create_effect(move || set_stored_item(key.as_str(), result.with(|c| (*c).clone())));
    result
}

