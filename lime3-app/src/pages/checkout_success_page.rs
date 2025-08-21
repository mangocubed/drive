use dioxus::prelude::*;

#[component]
pub fn CheckoutSuccessPage -> Element {
    rsx! {
        PageTitle { "Success" }

        h1 { class: "h1", "Success" }
    }
}