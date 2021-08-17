use yew::prelude::*;

pub fn render(id: &str) -> Html {
    html! {
        format!["Endpoint for game {}", id]
    }
}
