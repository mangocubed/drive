use dioxus::prelude::*;

use crate::components::PageTitle;

#[component]
pub fn HomePage() -> Element {
    rsx! {
        PageTitle { "Home" }

        h1 { class: "h1", "Home" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_a_heading_text() {
        let element = dioxus_ssr::render_element(HomePage());
        let expected = dioxus_ssr::render_element(rsx! {
            h1 { class: "h1", "Home" }
        });

        pretty_assertions::assert_str_eq!(element, expected);
    }
}
