use sycamore::prelude::*;


#[component(inline_props)]
pub fn ValueInput<G: Html>(    
    children: Children<G>,
    lable: String,
    value: Signal<f64>,) -> View<G> {
    let children = children.call();
    view! {
            span{(lable)}
            input(bind:valueAsNumber=value, type="number", min="0", step="0.1",maxlength="4",size="8")
            div{(children)}
    }
}

#[component(inline_props)]
pub fn ValueOutput<G: Html>(children: Children<G>, lable: String, value: ReadSignal<f64>) -> View<G> {
    let children = children.call();
    view! {
            span{(lable)}
            span{(((value.get()*100.0).round() / 100.0))}
            div{(children)}
    }
}

#[component(inline_props)]
pub fn MenuButtons<G: Html>(raw_visable: Signal<bool>, menu_visable: Signal<bool>) -> View<G> {
    view! {
        div(class="leaflet-bar leaflet-control"){
            a (class="map-button",href="#", title="Open Menu", on:click=move |_| {
                menu_visable.set(!menu_visable.get());
                raw_visable.set(false);
            }){"⚙"}
            a (class="map-button",href="#", title="Open Menu", on:click=move |_| {
                raw_visable.set(!raw_visable.get());
                menu_visable.set(false);
            }){"⚛"}
        }
    }
}
