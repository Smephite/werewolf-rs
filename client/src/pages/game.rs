use yew::prelude::*;

#[derive(Eq, PartialEq, Properties)]
pub struct Props {
    pub id: u64,
}
pub struct Game {
    game_id: u64,
}

impl Component for Game {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            game_id: ctx.props().id,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! { "Game" }
    }
}
