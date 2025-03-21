use arboard::Clipboard;
use std::str::FromStr;

use common::language::get_local_text;
use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Modifiers;
use futures::{channel::oneshot, StreamExt};
use kit::components::context_menu::{ContextItem, ContextMenu};

use kit::elements::{
    button::Button,
    input::{Input, Options, SpecialCharsAction, Validation},
    label::Label,
    tooltip::Tooltip,
};

use warp::error::Error;
use warp::{crypto::DID, logging::tracing::log};

use common::icons::outline::Shape as Icon;
use common::{
    state::{Action, Identity, State, ToastNotification},
    warp_runner::{MultiPassCmd, WarpCmd},
    STATIC_ARGS, WARP_CMD_CH,
};

#[allow(non_snake_case)]
pub fn AddFriend(cx: Scope) -> Element {
    let state = use_shared_state::<State>(cx)?;
    let clear_input = use_state(cx, || false);
    let friend_input = use_state(cx, String::new);
    let friend_input_valid = use_state(cx, || false);
    let request_sent = use_state(cx, || false);
    let error_toast: &UseState<Option<String>> = use_state(cx, || None);
    let add_in_progress = use_state(cx, || false);
    // used when copying the user's id to the clipboard
    let my_id: &UseState<Option<String>> = use_state(cx, || None);
    // Set up validation options for the input field
    let friend_validation = Validation {
        max_length: Some(56),
        min_length: Some(9), // Min amount of chars which is the short did (8 chars) + the hash symbol
        alpha_numeric_only: true,
        no_whitespace: true,
        ignore_colons: true,
        special_chars: Some((SpecialCharsAction::Allow, vec!['#'])),
    };

    if *clear_input.get() {
        friend_input.set(String::new());
        friend_input_valid.set(false);
        clear_input.set(false);
    }

    if *request_sent.get() {
        state
            .write()
            .mutate(Action::AddToastNotification(ToastNotification::init(
                "".into(),
                get_local_text("friends.request-sent"),
                None,
                2,
            )));
        request_sent.set(false);
    }

    if let Some(msg) = error_toast.get().clone() {
        state
            .write()
            .mutate(Action::AddToastNotification(ToastNotification::init(
                "".into(),
                msg,
                None,
                2,
            )));
        error_toast.set(None);
    }

    if let Some(id) = my_id.get().clone() {
        match Clipboard::new() {
            Ok(mut c) => {
                if let Err(e) = c.set_text(id) {
                    log::warn!("Unable to set text to clipboard: {e}");
                }
            }
            Err(e) => {
                log::warn!("Unable to create clipboard reference: {e}");
            }
        };
        state
            .write()
            .mutate(Action::AddToastNotification(ToastNotification::init(
                "".into(),
                get_local_text("friends.copied-did"),
                None,
                2,
            )));
        my_id.set(None);
    }

    let identity = state.read().get_own_identity();
    let short_id = identity.short_id();
    let did_key = identity.did_key();
    let username = identity.username();
    let short_name = format!("{}#{}", username, short_id);
    let short_name_context = short_name.clone();
    let add_friend_lable = if !state.read().ui.is_minimal_view() {
        get_local_text("uplink.add")
    } else {
        String::new()
    };

    let add_friend_icon = if !state.read().ui.is_minimal_view() {
        Icon::Plus
    } else {
        Icon::UserPlus
    };

    let ch = use_coroutine(cx, |mut rx: UnboundedReceiver<(String, Vec<Identity>)>| {
        to_owned![
            request_sent,
            error_toast,
            add_in_progress,
            clear_input,
            short_name
        ];
        async move {
            let warp_cmd_tx = WARP_CMD_CH.tx.clone();
            while let Some((id, outgoing_requests)) = rx.next().await {
                let id_to_compare = id.clone();

                //tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
                let (tx, rx) = oneshot::channel::<Result<(), warp::error::Error>>();
                if let Err(e) = warp_cmd_tx.send(WarpCmd::MultiPass(MultiPassCmd::RequestFriend {
                    id,
                    outgoing_requests,
                    rsp: tx,
                })) {
                    log::error!("failed to send warp command: {}", e);
                    add_in_progress.set(false);
                    // todo: should input be cleared here?
                    clear_input.set(true);
                    continue;
                }

                let res = rx.await.expect("failed to get response from warp_runner");
                add_in_progress.set(false);
                clear_input.set(true);
                match res {
                    Ok(_) => {
                        request_sent.set(true);
                    }
                    Err(e) => match e {
                        Error::CannotSendSelfFriendRequest => {
                            log::warn!("cannot add self: {}", e);
                            error_toast.set(Some(get_local_text("friends.cannot-add-self")));
                        }
                        Error::PublicKeyIsBlocked => {
                            log::warn!("add friend failed: {}", e);
                            error_toast.set(Some(get_local_text("friends.key-blocked")));
                        }
                        Error::FriendExist => {
                            log::warn!("add friend failed: {}", e);
                            error_toast.set(Some(get_local_text("friends.add-existing-friend")));
                        }
                        Error::FriendRequestExist => {
                            log::warn!("request already pending: {}", e);
                            error_toast.set(Some(get_local_text("friends.request-exist")));
                        }
                        _ => {
                            if id_to_compare == short_name {
                                log::warn!("cannot add self: {}", e);
                                error_toast.set(Some(get_local_text("friends.cannot-add-self")));
                            } else {
                                error_toast.set(Some(get_local_text("friends.add-failed")));
                                log::error!("add friend failed: {}", e);
                            }
                        }
                    },
                }
            }
        }
    });

    cx.render(rsx!(
        div {
            class: "add-friend",
            Label {
                text: get_local_text("friends.add"),
                aria_label: "add-friend-label".into(),
            },
            div {
                class: "body",
                ContextMenu {
                    key: "{context_key}",
                    id: "add-friend-input-context-menu".into(),
                    devmode: state.read().configuration.developer.developer_mode,
                    children: cx.render(rsx!(
                        Input {
                            placeholder: get_local_text("friends.placeholder"),
                            icon: Icon::MagnifyingGlass,
                            value: friend_input.get().clone(),
                            options: Options {
                                with_validation: Some(friend_validation),
                                // Do not replace spaces with underscores
                                replace_spaces_underscore: false,
                                // Show a clear button inside the input field
                                with_clear_btn: true,
                                clear_validation_on_no_chars: true,
                                // Use the default options for the remaining fields
                                ..Options::default()
                            },
                            disable_onblur: true,
                            loading: *add_in_progress.current(),
                            disabled: *add_in_progress.current(),
                            reset: clear_input.clone(),
                            onreturn: move |_| {
                                if !friend_input_valid.get() {
                                    return;
                                }
                                if STATIC_ARGS.use_mock {
                                    if let Ok(did) = DID::from_str(friend_input.get()) {
                                        let mut ident = Identity::default();
                                        ident.set_did_key(did);
                                        state.write().mutate(Action::SendRequest(ident));
                                    }
                                } else {
                                    add_in_progress.set(true);
                                    ch.send((friend_input.get().to_string(), state.read().outgoing_fr_identities()));
                                }
                            },
                            onchange: |(s, is_valid)| {
                                friend_input.set(s);
                                friend_input_valid.set(is_valid);
                            },
                            aria_label: "Add Someone Input".into()
                        }
                    )),
                    items: cx.render(rsx!(
                        ContextItem {
                            icon: Icon::ClipboardDocument,
                            aria_label: "friend-add-input-copy".into(),
                            text: get_local_text("uplink.copy-text"),
                            onpress: move |_| {
                                let text = friend_input.get().to_string();
                                match Clipboard::new() {
                                    Ok(mut c) => {
                                        if let Err(e) = c.set_text(text) {
                                            log::warn!("Unable to set text to clipboard: {e}");
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("Unable to create clipboard reference: {e}");
                                    }
                                };
                            }
                        },
                        ContextItem {
                            icon: Icon::ClipboardDocument,
                            aria_label: "friend-add-input-paste".into(),
                            text: get_local_text("uplink.paste"),
                            onpress: move |_| {
                                match Clipboard::new() {
                                    Ok(mut c) => {
                                        let text = c.get_text().map_err(|e| {
                                             log::error!("Unable to get clipboard content: {e}");
                                             e
                                        }).unwrap_or_default();
                                        friend_input.set(text);
                                    }
                                    Err(e) => {
                                        log::warn!("Unable to create clipboard reference: {e}");
                                    }
                                };
                            }
                        }
                    ))
                }
                Button {
                    icon: add_friend_icon,
                    text: add_friend_lable.to_string(),
                    disabled: !friend_input_valid.get(),
                    onpress: move |_| {
                        if STATIC_ARGS.use_mock {
                            if let Ok(did) = DID::from_str(friend_input.get()) {
                                let mut ident = Identity::default();
                                ident.set_did_key(did);
                                state.write().mutate(Action::SendRequest(ident));
                            }
                        } else {
                            add_in_progress.set(true);
                            ch.send((friend_input.get().to_string(), state.read().outgoing_fr_identities()));
                        }
                    },
                    aria_label: "Add Someone Button".into()
                },
                div {
                    ContextMenu {
                        id: String::from("copy-id-context-menu"),
                        items: cx.render(rsx!(
                            ContextItem {
                                icon: Icon::UserCircle,
                                text: get_local_text("settings-profile.copy-id"),
                                aria_label: "copy-id-context".into(),
                                onpress: move |_| {
                                    match Clipboard::new() {
                                        Ok(mut c) => {
                                            if let Err(e) = c.set_text(short_name_context.clone()) {
                                                log::warn!("Unable to set text to clipboard: {e}");
                                            }
                                        },
                                        Err(e) => {
                                            log::warn!("Unable to create clipboard reference: {e}");
                                        }
                                    };
                                    state
                                        .write()
                                        .mutate(Action::AddToastNotification(ToastNotification::init(
                                            "".into(),
                                            get_local_text("friends.copied-did"),
                                            None,
                                            2,
                                        )));
                                }
                            }
                            ContextItem {
                                icon: Icon::Key,
                                text: get_local_text("settings-profile.copy-did"),
                                aria_label: "copy-id-context".into(),
                                onpress: move |_| {
                                    match Clipboard::new() {
                                        Ok(mut c) => {
                                            if let Err(e) = c.set_text(did_key.to_string()) {
                                                log::warn!("Unable to set text to clipboard: {e}");
                                            }
                                        },
                                        Err(e) => {
                                            log::warn!("Unable to create clipboard reference: {e}");
                                        }
                                    };
                                    state
                                        .write()
                                        .mutate(Action::AddToastNotification(ToastNotification::init(
                                            "".into(),
                                            get_local_text("friends.copied-did"),
                                            None,
                                            2,
                                        )));
                                }
                            }
                        )),
                        Button {
                            aria_label: "Copy ID".into(),
                            icon: Icon::ClipboardDocument,
                            onpress: move |mouse_event: MouseEvent| {
                                if mouse_event.modifiers() != Modifiers::CONTROL {
                                    match Clipboard::new() {
                                        Ok(mut c) => {
                                            if let Err(e) = c.set_text(short_name.clone()) {
                                                log::warn!("Unable to set text to clipboard: {e}");
                                            }
                                        },
                                        Err(e) => {
                                            log::warn!("Unable to create clipboard reference: {e}");
                                        }
                                    };
                                    state
                                        .write()
                                        .mutate(Action::AddToastNotification(ToastNotification::init(
                                            "".into(),
                                            get_local_text("friends.copied-did"),
                                            None,
                                            2,
                                        )));
                                }
                            },
                            tooltip: cx.render(rsx!(Tooltip{
                                text: get_local_text("settings-profile.copy-id")
                            }))
                        }
                    }
                }
            }
        }
    ))
}
