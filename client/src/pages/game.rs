use yew::prelude::*;

pub fn render(id: &String) -> Html {
    html! {
        format!["Endpoint for game {}", id]
    }
}
