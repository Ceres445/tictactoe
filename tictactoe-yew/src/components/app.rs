use yew::prelude::*;

use crate::components::pagelayout::{Footer, Header, PageLayout};
use crate::components::tictactoe::TicTacToe;

use tictactoe_library::app::App;

#[function_component(YewApp)]
pub fn app() -> Html {
    let app = App::new();
    html! {
        <PageLayout>
            <Header />
            <main><TicTacToe app = {app}/></main>
            <Footer />
        </PageLayout>
    }
}
