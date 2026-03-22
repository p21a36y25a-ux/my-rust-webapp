use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
}

#[function_component(App)]
pub fn app() -> Html {
    let users = use_state(Vec::<User>::new);

    {
        let users = users.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::get("http://localhost:8080/api/users").send().await {
                    if let Ok(list) = response.json::<Vec<User>>().await {
                        users.set(list);
                    }
                }
            });

            || ()
        });
    }

    html! {
        <div>
            <h1>{ "My Rust Webapp" }</h1>
            <ul>
                { for users.iter().map(|u| html! { <li>{ format!("{} <{}>", u.name, u.email) }</li> }) }
            </ul>
        </div>
    }
}
