use yew::prelude::*;
use yew_router::prelude::*;
use crate::pages;
#[derive(::yew_router::Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/create")]
    CreateGame,
    #[at("/game/:id")]
    Game{id: String},
    #[at("/")]
    Home,
    #[at("/404")]
    #[not_found]
    NotFound,
}

enum Msg {}

struct Model {}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                { self.view_nav() }
                <main>
                    <Router<Route> render=Router::render(switch) />
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
        }
    }
}


impl Model {
    fn view_nav(&self) -> Html {        
        html! {
            <nav class="navbar is-primary" role="navigation">
                <div class="navbar-brand">
                    {go_to(Route::Home, html!{
                        <>
                        <span class="is-size-3" style="padding-right: 0px">{"werewolf"}</span><span style="padding-left: 0px" class="is-size-5">{"-rs"}</span>
                        </>
                    }, vec!["navbar-item", "no-hover"])}
                </div>
            </nav>
        }
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => {
            pages::home::render()
        },
        Route::Game { id } => {
            pages::game::render(id)
        },
        Route::CreateGame => {
            pages::create_game::render()
        }
        Route::NotFound => {
            pages::not_found::render()
        }
    }
}

pub fn go_to(route: Route, html: Html, classes: Vec<&str>) -> Html {
    html!{
        <Link<Route> route=route classes=classes!(classes.iter().map(|&s| String::from(s)).collect::<Vec<String>>())>{html}</Link<Route>>
    }
}

pub fn start() {
    let document = yew::utils::document();
    let element = document.query_selector("#app").unwrap().unwrap();
    yew::start_app_in_element::<Model>(element);
}
