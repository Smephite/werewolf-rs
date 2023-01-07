mod pages;
mod route;

use route::Model;

fn main() {
    yew::start_app::<Model>();
}
