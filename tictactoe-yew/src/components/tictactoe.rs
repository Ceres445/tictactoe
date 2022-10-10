use yew::prelude::*;

use tictactoe_library::app::{App, AppState, Menu};

use crate::components::menu::{GameMenu, StartMenu};

#[derive(Properties, PartialEq)]
pub struct AppProps {
    #[prop_or_default]
    children: Children,
    pub app: App,
}

#[function_component(TicTacToe)]
pub fn tictactoe_app(props: &AppProps) -> Html {
    let app = &props.app;
    match &app.state {
        AppState::Menu(menu, _) => match menu {
            Menu::Start => {
                html! {
                    <StartMenu />
                }
            }
            Menu::Game => {
                html! {
                    <GameMenu />
                }
            }
        },
        AppState::Playing(_) => {
            html! {
                // <Game game_state={game_state} />
                <div> </div>
            }
        }
        _ => {
            html! {
                <div> </div>
            }
        }
    }
}
