use dioxus::prelude::*;

mod components;
mod layout;
mod pages;
mod routes;

use routes::Routes;

const STYLE_CSS: Asset = asset!("assets/style.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: STYLE_CSS }

        Router::<Routes> {}
    }
}
