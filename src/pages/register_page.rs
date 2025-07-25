use dioxus::prelude::*;

use crate::forms::Form;
use crate::components::PageTitle;

#[component]
pub fn RegisterPage() -> Element {
    rsx! {
        PageTitle { "Register" }

        h1 { class: "h1", "Register" }

        Form { }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_has_a_heading_text() {
        let element = dioxus_ssr::render_element(RegisterPage());
        let expected = dioxus_ssr::render_element(rsx! {
            h1 { class: "h1", "Register" }
        });

        pretty_assertions::assert_str_eq!(element, expected);
    }
