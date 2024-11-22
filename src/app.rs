use std::{collections::BTreeMap, net::IpAddr};

use ev::MouseEvent;
use leptos::leptos_dom::ev::SubmitEvent;
use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::HtmlDialogElement;

use crate::model::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    type Store;

    #[wasm_bindgen(method)]
    async fn entries(this: &Store) -> JsValue;
}

// Theoretically this should work?
async fn store_load(path: &str) -> f64 {
    invoke("plugin:store|load",
        serde_wasm_bindgen::to_value(&Load {
            path,
            options: Some(LoadOptions { autoSave: Some(true) })
        }).expect("failed to serialize args")
    ).await.as_f64().expect("failed to get store rid")
}

async fn store_save_string(rid: f64, key: &str, value: &str) {
    invoke("plugin:store|set",
        serde_wasm_bindgen::to_value(&StorePluginSetArgs {
            rid, key, value
        }).expect("failed to serialize args")
    ).await;
}

async fn store_get_string(rid: f64, key: &str) -> Option<String> {
    let val = invoke("plugin:store|get",
        serde_wasm_bindgen::to_value(&StorePluginGetArgs {
            rid, key
        }).expect("failed to serialize args")
    ).await;
    logging::log!("Got: {:?}", &val);
    if let Ok((l, _r)) = serde_wasm_bindgen::from_value::<(String, bool)>(val) {
        Some(l)
    } else {
        None
    }
}

fn map_append(option: &str) -> &str {
    match option {
        "LF" => "\n",
        "CR" => "\r",
        "CRLF" => "\r\n",
        _ => "",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppData {
    pub last_address: Option<String>,
    pub last_port: Option<String>,
}

#[component]
pub fn Sidebar(
    children: Children,
    open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div>
            <div
                style="position: fixed; width: 100%; height: 100%; top: 0; left: 0;"
                style:background-color=move || format!("rgba(0,0,0,{})", if open.get() { 0.5 } else { 0.0 })
                style:visibility=move || if open.get() { "visible" } else { "hidden" }
                style:transition="all 0.5s ease-out allow-discrete"
                on:click=move|_| open.set(false)
            ></div>
            <div
                class="light-contrast"
                on:click=move|_| open.set(false) 
                style="position: fixed; width: 80%; height: 100%; top: 0; left: 0;"
                style:left=move || if open.get() { "0" } else { "-80%" }
                style:transition="all 0.5s ease-out allow-discrete"
            >
                <div style="width: 100%; height: 100%;" on:click=move|e| e.stop_propagation()>
                    {children()}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let address = create_rw_signal(String::new());
    let port = create_rw_signal(String::new());
    let message = create_rw_signal(String::new());
    let append = create_rw_signal(String::new());
    let prepend = create_rw_signal(String::new());
    let prepend_enabled = create_rw_signal(false);

    let saved = create_rw_signal(BTreeMap::<String, String>::new());
    let command_name = create_rw_signal(String::new());

    let update_address = move |ev| address.set(event_target_value(&ev));
    let update_port = move |ev| port.set(event_target_value(&ev));
    let update_message = move |ev| message.set(event_target_value(&ev));
    let update_command_name = move |ev| command_name.set(event_target_value(&ev));
    let update_prepend = move |ev| prepend.set(event_target_value(&ev));

    let update_append = move |ev| {
        let updated = event_target_value(&ev);
        append.set(updated.clone());

        // See if this works?
        spawn_local(async move {
            let store = store_load("store.json").await;
            store_save_string(store, "last_append", &updated).await;
            logging::log!("Last append saved as {updated}");
        });
    };

    let connect = move |ev: SubmitEvent| {
        ev.prevent_default();
        
        if address.get().is_empty() {
            window().alert_with_message("Missing address").unwrap();
            return;
        }
        if let Err(e) =  address.get().parse::<IpAddr>() {
            window().alert_with_message(&format!("Invalid IP address; error: {e}")).unwrap();
            return;
        }
        if port.get().is_empty() {
            window().alert_with_message("Missing port number").unwrap();
            return;
        }
        let port: i32 = port.get().parse().expect("failed to get number from form input");
        let address = address.get();
        

        spawn_local( {
            let address = address.clone();
            async move {
                // TODO: Use plugin directly instead
                invoke("store_set", serde_wasm_bindgen::to_value(&StoreSetArgs { path: "store.json", key: "last_address", value: &address }).unwrap()).await;
                logging::log!("Saved address: {address}");
                invoke("store_set", serde_wasm_bindgen::to_value(&StoreSetArgs { path: "store.json", key: "last_port", value: &port.to_string() }).unwrap()).await;
                logging::log!("Saved port: {port}");
            }
        });

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&ConnectArgs { address: &address, port }).unwrap();
            let resp = invoke("plugin:tcp-client|connect", args).await;
            let result: ConnectResponse = serde_wasm_bindgen::from_value(resp).unwrap();
            // too lazy for proper popup
            if let Some(err) = result.error {
                window().alert_with_message(&format!("Error connecting to server: {err}")).unwrap();
            } else {
                window().alert_with_message("Connected to server!").unwrap();
            }
        });
    };

    let disconnect = move |ev: ev::MouseEvent| {
        ev.prevent_default();
        spawn_local(async move {
            invoke("plugin:tcp-client|disconnect", JsValue::null()).await;
        });
    };

    let transmit_message = move |message: String| {
        let front = prepend_enabled.get().then(|| prepend.get()).unwrap_or_default();
        let message = front + &message + map_append(&append.get());
        logging::log!("would send {message}");
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&TransmitArgs { message: &message }).unwrap(); 
            let resp = invoke("plugin:tcp-client|transmit", args).await;
            let result: TransmitResponse = serde_wasm_bindgen::from_value(resp).unwrap();
            if let Some(err) = result.error {
                window().alert_with_message(&err).unwrap();
            }
        });
    };

    let transmit = move |ev: SubmitEvent| {
        ev.prevent_default();
        transmit_message(message.get());
    };

    let save_command = create_action(move |(name, message): &(String, String)| {
        let name = name.clone();
        let message = message.clone();
        saved.update(|saved| { saved.insert(name.clone(), message.clone()); });
        async move {
            let store = invoke("plugin:store|load", serde_wasm_bindgen::to_value(&Load { path: "commands.json", options: Some(LoadOptions { autoSave: Some(true) }) }).unwrap()).await;
            logging::log!("{store:?}, {:?}", store.js_typeof());
            // let store = store.unchecked_into::<Store>();
            let store = store.as_f64().unwrap();
            invoke("plugin:store|set", serde_wasm_bindgen::to_value(&StorePluginSetArgs {
                rid: store,
                key: &name,
                value: &message,
            }).unwrap()).await;
        }
    });

    let save = move |ev: SubmitEvent| {
        ev.prevent_default();
        let name = command_name.get();
        if name.is_empty() {
            window().alert_with_message(&format!("Command name field is empty!")).unwrap();
            return;
        }
        if saved.get().contains_key(&name) {
            window().alert_with_message(&format!("Command with name '{name}' already exists")).unwrap();
            return;
        }
        if message.get().is_empty() {
            window().alert_with_message(&format!("Current message is empty!")).unwrap();
            return;
        }
        save_command.dispatch((name, message.get()));
    };

    let del = create_action(move |name: &String| {
        let name = name.clone();
        async move {
            let args = serde_wasm_bindgen::to_value(&ConfirmArgs { msg: &format!("Delete command '{name}'?") }).unwrap();
            let resp = invoke("ask", args).await;
            if let Ok(true) = serde_wasm_bindgen::from_value(resp) {
                saved.update(|saved| { saved.remove(&name); });

                let store = invoke("plugin:store|load", serde_wasm_bindgen::to_value(&Load { path: "commands.json", options: Some(LoadOptions { autoSave: Some(true) }) }).unwrap()).await;
                logging::log!("{store:?}, {:?}", store.js_typeof());
                let store = store.as_f64().unwrap();
                invoke("plugin:store|delete", serde_wasm_bindgen::to_value(&StorePluginDeleteArgs {
                    rid: store,
                    key: &name,
                }).unwrap()).await;
                return true;
            }
            return false;
        }
    });

    let delete = move |ev: MouseEvent| {
        ev.prevent_default();

        let name = event_target_value(&ev);
        del.dispatch(name);
    };
    let send = move |ev| {
        let name = event_target_value(&ev);
        if let Some(message) = saved.get().get(&name).cloned() {
            transmit_message(message);
        } else {
            window().alert_with_message(&format!("No command with name '{name}' found")).unwrap();
        }
    };

    spawn_local(async move {
        let resp = invoke("initialize", JsValue::null()).await;
        let initial_state: AppData = serde_wasm_bindgen::from_value(resp).unwrap();
        let last_address = initial_state.last_address.unwrap_or_default();
        let last_port = initial_state.last_port.unwrap_or_default();
        address.set(last_address.clone());
        port.set(last_port.clone());
        leptos::logging::log!("Initialize function has been called");
        leptos::logging::log!("Last address: {last_address}");
        leptos::logging::log!("Last port: {last_port}");

        // Load saved commands
        let store = invoke("plugin:store|load", serde_wasm_bindgen::to_value(&Load { path: "commands.json", options: Some(LoadOptions { autoSave: Some(true) }) }).unwrap()).await;
        logging::log!("{store:?}, {:?}", store.js_typeof());
        let store = store.as_f64().unwrap();
        let entries = invoke("plugin:store|entries", serde_wasm_bindgen::to_value(&EntriesArgs { rid: store }).unwrap()).await;
        let entries = serde_wasm_bindgen::from_value::<Vec<(String, String)>>(entries).unwrap();
        for (name, val) in entries.iter() {
            logging::log!("Name: {name}, val: {val}");
            saved.update(|saved| {
                saved.insert(name.clone(), val.clone());
            });
        }
        logging::log!("Entries: {entries:?}");
        
        let store = store_load("store.json").await;
        let last_append = store_get_string(store, "last_append").await.unwrap_or("CRLF".into());
        logging::log!("Last append: {last_append}");
        append.set(last_append);
        let last_prepend = store_get_string(store, "last_prepend").await.unwrap_or_default();
        prepend.set(last_prepend);
        let last_prepend_enabled = store_get_string(store, "last_prepend_enabled")
            .await.map_or(false, |s| s.to_lowercase() == "true");
        prepend_enabled.set(last_prepend_enabled);
    });

    let edit_popup = create_node_ref::<html::Dialog>();
    let edit_input = create_node_ref::<html::Input>();
    let prepend_enable_ref = create_node_ref::<html::Input>();

    create_effect(move |_| {
        logging::log!("this should print...?");
        if let Some(input) = prepend_enable_ref.get() {
            input.set_checked(prepend_enabled.get());
        }
    });

    let show_popup =move |ev| {
        let name = event_target_value(&ev);
        if let Some(dialog) = edit_popup.get() {
            dialog.set_attribute("edit-target", &name).expect("failed to set edit-target");
            edit_input.get().unwrap().set_value(&saved.get().get(&name).cloned().unwrap_or_default());
            dialog.show_modal().unwrap();
        }
    };

    let sidebar_open = create_rw_signal(false);

    view! {
        <main class="container" style="gap: 4px; height: 90vh">
            <h1>"TCP Commander"</h1>

            <button
                style="position: fixed; left: 0; top: 0;"
                on:click=move|_| sidebar_open.update(|v| *v = !*v)
            >
                "⚙️"
            </button>
            <Sidebar open=sidebar_open>
                <div>
                    <h2>"Options"</h2>
                </div>
                <div>
                    <p>"Append to Messages"</p>
                    <select on:input=update_append>
                        // Dynamic for responding to initialization
                        <For
                            each=move||["None", "LF", "CR", "CRLF"].map(|s| (s, s == append.get()))
                            key=|(option, selected)| (option.to_string(), *selected)
                            children=|(option, selected)| {
                                if selected {
                                    view! { <option value={option} selected="selected">{option}</option> }
                                } else {
                                    view! { <option value={option}>{option}</option> }
                                }
                            }
                        />
                    </select>
                </div>
                <div>
                    <p>"Prepend to Messages"</p>
                    <div class="row"
                        style:align-items="center"
                    >
                        <form on:submit=move |e| {
                            e.prevent_default();
                            spawn_local(async move {
                                let store = store_load("store.json").await;
                                store_save_string(store, "last_prepend", &prepend.get()).await;
                            });
                        }>
                            <input
                                on:input=update_prepend
                                // Might need to use a different event
                                on:blur=move |_| {
                                    spawn_local(async move {
                                        let store = store_load("store.json").await;
                                        store_save_string(store, "last_prepend", &prepend.get()).await;
                                    })
                                }
                                value=move|| prepend.get()
                            />
                        </form>
                        <label for="prepend-enable">
                            <input
                                type="checkbox"
                                name="prepend-enable"
                                id="prepend-enable"
                                _ref = prepend_enable_ref
                                on:input=move |e| {
                                    let checkbox = event_target::<web_sys::HtmlInputElement>(&e);
                                    prepend_enabled.set(checkbox.checked());
                                    spawn_local(async move {
                                        let store = store_load("store.json").await;
                                        store_save_string(store, "last_prepend_enabled", &format!("{}", checkbox.checked())).await;
                                    })
                                }
                            />
                            "Enable"
                        </label>
                    </div>
                </div>
            </Sidebar>

            <form on:submit=connect>
                <div class="row">
                    <button type="submit">"Connect"</button>
                    <button on:click=disconnect>"Disconnect"</button>
                </div>
                <div class="row">
                    <input
                        class="pair-input"
                        id="address-input"
                        placeholder="Address..."
                        style:width="50%"
                        on:input=update_address
                        value={move || address.get()}
                    />
                    <input
                        class="pair-submit"
                        id="port-input"
                        type="number"
                        placeholder="Port"
                        min="1024" max="65535"
                        on:input=update_port
                        value={move || port.get()}
                    />
                </div>
            </form>
            <div style="display: flex; flex-direction: column;">
                <form class="row" on:submit=transmit>
                    <input
                        class="pair-input"
                        id="message-input"
                        placeholder="Message"
                        style:width="50%"
                        on:input=update_message
                    />
                    <button class="pair-submit" type="submit">"Transmit"</button>
                </form>
                <form class="row" on:submit=save>
                    <input
                        class="pair-input"
                        id="name-input"
                        placeholder="Command Name"
                        on:input=update_command_name
                    />
                    <button class="pair-submit" type="submit">"Save"</button>
                </form>
            </div>
            <div style="overflow-y: scroll; display: flex; flex-direction: column; margin-top: auto;">
                <For
                    each=move || saved.get()
                    key=|command| command.0.clone() + &command.1
                    children=move |command|  {
                        let name = command.0;
                        let message = command.1;
                        view ! {
                            <div class="light-contrast" style="padding: 10px; border-radius: 10px; margin: 10px;">
                                <p>{&name}</p>
                                <p>{&message}</p>
                                <div class="row">
                                    <button value={&name} on:click=send style="margin: 4px">"Send"</button>
                                    <button value={&name} on:click=delete style="margin: 4px">"Delete"</button>
                                    <button value={&name} on:click=show_popup style="margin: 4px">"Edit"</button>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
            <dialog class="light-contrast" _ref=edit_popup id="popup" on:click=move|e| {
                if event_target::<HtmlDialogElement>(&e) == *edit_popup.get().unwrap().unchecked_ref() {
                    edit_popup.get().unwrap().close();
                }
            }>
                <div on:click=move|e| e.stop_propagation()>
                    <p>{edit_popup.get().unwrap().get_attribute("edit-target").unwrap_or_default()}</p>
                    <form on:submit=move|e| {
                        e.prevent_default();
                        let action = e.submitter().unwrap().get_attribute("value").unwrap();
                        let target = edit_popup.get().unwrap().get_attribute("edit-target").expect("edit-target not set");
                        match action.as_str() {
                            "submit" => {
                                let val = edit_input.get().expect("input should exist").value();
                                if Some(&val) == saved.get().get(&target) {
                                    window().alert_with_message("Message unchanged!").unwrap();
                                } else {
                                    save_command.dispatch((target, val));
                                    edit_popup.get().unwrap().close();
                                }
                            }
                            _ => edit_popup.get().unwrap().close(),
                        }
                    }>
                        <input autofocus type="text" id="command-edit" _ref=edit_input/>
                        <button value="submit">"Confirm"</button>
                        <button value="cancel">"Cancel"</button>
                    </form>
                </div>
            </dialog>
        </main>
    }
}
