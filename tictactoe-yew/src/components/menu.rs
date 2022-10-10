use yew::prelude::*;

pub enum Msg {
    One,
    Two,
    Three,
}

pub struct StartMenu {
    row: u8,
}

impl Component for StartMenu {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        StartMenu { row: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::One => {
                self.row = 0;
            }
            Msg::Two => {
                self.row = 1;
            }
            Msg::Three => {
                self.row = 2;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
         <div class="btn-group btn-group-vertical">
            <button class = "btn" id="1" onclick={link.callback(|_| Msg::One)}>{"Play Against Human" }</button>
            <button class = "btn" id="2" onclick={link.callback(|_| Msg::Two)}>{ "Play Against Random Computer "}</button>
            <button class = "btn" id="3" onclick={link.callback(|_| Msg::Three)}>{ "Play Against Smart Computer "}</button>

            <p> {{self.row}} </p>
            </div>
        }
    }
}
// #[function_component(StartMenu)]
// pub fn start_menu() -> Html {
//     let
//     html! {
//         <div class="btn-group btn-group-vertical">
//             <button class="btn"  on_click = {link.callback(|_| Msg::One)}>{{ "Play Against Human" }}</button>
//             <button class="btn">{{ "Play Against Random Computer "}}</button>
//             <button class="btn">{{ "Play Against Smart Computer "}}</button>
//         </div>
//     }
// }

#[function_component(GameMenu)]
pub fn game_menu() -> Html {
    html! {
        <div class="btn-group btn-group-vertical">
        <button class="btn">{{ "Resume Game" }}</button>
        <button class="btn">{{ "New Game"}}</button>
        <button class="btn">{{ "Quit"}}</button>

        </div>
    }
}
