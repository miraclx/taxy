use crate::auth::use_ensure_auth;
use crate::components::breadcrumb::Breadcrumb;
use crate::pages::Route;
use crate::store::PortStore;
use crate::utils::convert_multiaddr;
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::{
    id::ShortId,
    port::{PortEntry, PortStatus, SocketState},
};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(PortList)]
pub fn post_list() -> Html {
    use_ensure_auth();

    let (ports, dispatcher) = use_store::<PortStore>();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_list().await {
                    let mut statuses = HashMap::new();
                    for entry in &res {
                        if let Ok(status) = get_status(entry.id).await {
                            statuses.insert(entry.id, status);
                        }
                    }
                    dispatcher.set(PortStore {
                        entries: res,
                        statuses,
                    });
                }
            });
        },
        (),
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let new_port_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::NewPort);
    });

    let list = ports.entries.clone();
    let active_index = use_state(|| -1);
    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>
            if list.is_empty() {
                <ybc::Hero body_classes="has-text-centered" body={
                    html! {
                    <p class="title has-text-grey-lighter">
                        {"No Items"}
                    </p>
                    }
                } />
            }
            <div class="list has-visible-pointer-controls">
            { list.into_iter().enumerate().map(|(i, entry)| {
                let navigator = navigator.clone();
                let active_index = active_index.clone();
                let status = ports.statuses.get(&entry.id).cloned().unwrap_or_default();

                let id = entry.id;
                let navigator_cloned = navigator.clone();
                let log_onclick = Callback::from(move |_|  {
                    let id = id;
                    navigator_cloned.push(&Route::PortLogView {id});
                });

                let id = entry.id;
                let navigator_cloned = navigator.clone();
                let config_onclick = Callback::from(move |_|  {
                    let id = id;
                    navigator_cloned.push(&Route::PortView {id});
                });

                let delete_onmousedown = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                });
                let id = entry.id;
                let delete_onclick = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                    if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                        let id = id;
                        wasm_bindgen_futures::spawn_local(async move {
                            let _ = delete_port(id).await;
                        });
                    }
                });

                let reset_onmousedown = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                });
                let id = entry.id;
                let reset_onclick = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                    if gloo_dialogs::confirm(&format!("Are you sure to reset {id}?\nThis operation closes all existing connections. ")) {
                        let id = id;
                        wasm_bindgen_futures::spawn_local(async move {
                            let _ = reset_port(id).await;
                        });
                    }
                });

                let active_index_cloned = active_index.clone();
                let dropdown_onclick = Callback::from(move |_|  {
                    active_index_cloned.set(if *active_index_cloned == i as i32 {
                        -1
                    } else {
                        i as i32
                    });
                });
                let active_index_cloned = active_index.clone();
                let dropdown_onfocusout = Callback::from(move |_|  {
                    active_index_cloned.set(-1);
                });
                let is_active = *active_index == i as i32;
                let title = if entry.port.name.is_empty() {
                    entry.id.to_string()
                } else {
                    entry.port.name.clone()
                };
                let (status_text, tag) = match status.state.socket {
                    SocketState::Listening => ("Listening", "is-success"),
                    SocketState::AddressAlreadyInUse => ("Address Already In Use", "is-danger"),
                    SocketState::PermissionDenied => ("Permission Denied", "is-danger"),
                    SocketState::AddressNotAvailable => ("Address Not Available", "is-danger"),
                    SocketState::Error => ("Error", "is-danger"),
                    SocketState::Unknown => ("Unknown", "is-light"),
                };
                let (protocol, addr) = convert_multiaddr(&entry.port.listen);
                html! {
                    <div class="list-item">
                        <div class="list-item-content">
                            <div class="list-item-title">{title}</div>
                            <div class="list-item-description field is-grouped mt-1">
                                <span class={classes!("tag", "is-success", "mr-2", tag)}>{status_text}</span>
                                <span class="tags has-addons">
                                    <span class="tag is-dark">{protocol}</span>
                                    <span class="tag is-info">{addr}</span>
                                </span>
                            </div>
                        </div>

                        <div class="list-item-controls">
                            <div class="buttons is-right">
                                <button type="button" data-tooltip="Logs" class="button" onclick={log_onclick}>
                                    <span class="icon is-small">
                                        <ion-icon name="receipt"></ion-icon>
                                    </span>
                                </button>

                                <button type="button" class="button" data-tooltip="Configs" onclick={config_onclick}>
                                    <span class="icon is-small">
                                        <ion-icon name="settings"></ion-icon>
                                    </span>
                                </button>

                                <div class={classes!("dropdown", "is-right", is_active.then_some("is-active"))}>
                                    <div class="dropdown-trigger" onfocusout={dropdown_onfocusout}>
                                        <button type="button" class="button" onclick={dropdown_onclick}>
                                            <span class="icon is-small">
                                                <ion-icon name="ellipsis-horizontal"></ion-icon>
                                            </span>
                                        </button>
                                    </div>
                                    <div class="dropdown-menu" id="dropdown-menu" role="menu">
                                        <div class="dropdown-content">
                                            <a class="dropdown-item" onmousedown={reset_onmousedown} onclick={reset_onclick}>
                                                <span class="icon-text">
                                                    <span class="icon">
                                                        <ion-icon name="refresh"></ion-icon>
                                                    </span>
                                                    <span>{"Reset"}</span>
                                                </span>
                                            </a>
                                            <a class="dropdown-item has-text-danger	" onmousedown={delete_onmousedown} onclick={delete_onclick}>
                                                <span class="icon-text">
                                                    <span class="icon">
                                                        <ion-icon name="trash"></ion-icon>
                                                    </span>
                                                    <span>{"Delete"}</span>
                                                </span>
                                            </a>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            }).collect::<Html>() }
            </div>
            <ybc::CardFooter>
                <a class="card-footer-item" onclick={new_port_onclick}>
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="add"></ion-icon>
                    </span>
                    <span>{"New Port"}</span>
                    </span>
                </a>
            </ybc::CardFooter>
            </ybc::Card>
        </>
    }
}

async fn get_list() -> Result<Vec<PortEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports"))
        .send()
        .await?
        .json()
        .await
}

async fn get_status(id: ShortId) -> Result<PortStatus, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports/{id}/status"))
        .send()
        .await?
        .json()
        .await
}

async fn delete_port(id: ShortId) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/ports/{id}"))
        .send()
        .await?;
    Ok(())
}

async fn reset_port(id: ShortId) -> Result<(), gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports/{id}/reset"))
        .send()
        .await?;
    Ok(())
}
