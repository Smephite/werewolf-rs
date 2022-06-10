use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{game::Game, home::Home, not_found::NotFound};

#[derive(::yew_router::Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/game/:id")]
    Game { id: u64 },
    #[at("/404")]
    #[not_found]
    NotFound,
    #[at("/")]
    Home,
}

pub struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _: &yew::Context<Self>) -> yew::Html {
        html!(
            <>
                {self.view_nav()}
                <main>
                    <Router<Route> render={Router::render(switch)} />
                </main>
                <footer class="footer">
                    <div class="content has-text-centered">
                        <p>
                            <strong><a href="https://github.com/Smephite/werewolf-rs">{"werewolf-rs"}</a></strong>
                            {" was made with ❤️ and 🍺"}
                        </p>
                    </div>
                </footer>
            </>
        )
    }
}

impl Model {
    fn view_nav(&self) -> Html {
        // TODO Add link to home
        html! {
            <nav class="navbar is-primary" role="navigation">
                <div class="navbar-brand">
                    <Link<Route> classes={classes!("navbar-item", "no-hover")} route={Route::Home}>
                        <>
                        <span class="is-size-3" style="padding-right: 0px">{"werewolf"}</span><span style="padding-left: 0px" class="is-size-5">{"-rs"}</span>
                        </>
                    </Link<Route>>
                </div>
            </nav>
        }
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => {
            html! { <Home /> }
        }
        Route::Game { id } => {
            html! { <Game id= { *id } />}
        }
        Route::NotFound => {
            html! { <NotFound /> }
        }
    }
}
