use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use floem::common::{alert, card_styles, create_icon, nav_button};
use floem::event::{Event, EventListener, EventPropagation};
use floem::ext_event::create_signal_from_tokio_channel;
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::CursorStyle;
use floem::taffy::{AlignItems, FlexDirection, JustifyContent};
use floem::text::Weight;
use floem::views::{
    button, container, dyn_container, dyn_stack, empty, h_stack, img, label, scroll, stack, svg,
    tab, text_input, v_stack, virtual_list, virtual_stack, VirtualDirection, VirtualItemSize,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use stunts_engine::editor::{Editor, Viewport};
use uuid::Uuid;
// use views::buttons::{nav_button, option_button, small_button};
// use winit::{event_loop, window};
use wgpu::util::DeviceExt;

use floem::context::PaintState;
// use floem::floem_reactive::SignalGet;
use floem::reactive::{SignalGet, SignalUpdate};
use floem::views::text;
use floem::views::Decorators;
use floem::IntoView;
use floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::EditorState;
use crate::helpers::projects::{get_projects, ProjectInfo};
use crate::helpers::utilities::{
    clear_auth_token, create_project_state, fetch_subscription_details, load_auth_token,
    load_project_state, save_auth_token, AuthState, AuthToken, SubscriptionDetails, API_URL,
};
// use crate::helpers::projects::{get_projects, ProjectInfo};
// use crate::helpers::websocket::WebSocketManager;

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize, Clone)]
struct LoginResponse {
    jwtData: JwtData,
}

#[derive(Deserialize, Clone)]
struct JwtData {
    token: String,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    expiry: Option<chrono::DateTime<chrono::Utc>>,
}

pub fn project_item(
    project_info: ProjectInfo,
    sortable_items: RwSignal<Vec<ProjectInfo>>,
    project_label: String,
    icon_name: &'static str,
) -> impl IntoView {
    h_stack((
        svg(create_icon(icon_name))
            .style(|s| s.width(24).height(24).color(Color::BLACK))
            .style(|s| s.margin_right(7.0)),
        label(move || project_label.to_string()),
    ))
    .style(|s| {
        s.width(260.0)
            .border_radius(15.0)
            .align_items(AlignItems::Center)
            .justify_start()
            .padding_vert(8)
            .background(Color::rgb(255.0, 255.0, 255.0))
            .border_bottom(1)
            .border_color(Color::rgb(200.0, 200.0, 200.0))
            .hover(|s| {
                s.background(Color::rgb(100.0, 100.0, 100.0))
                    .cursor(CursorStyle::Pointer)
            })
            .active(|s| s.background(Color::rgb(237.0, 218.0, 164.0)))
    })
}

pub fn project_browser(
    editor_state: Arc<Mutex<EditorState>>,
    editor: std::sync::Arc<Mutex<Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl View {
    let projects = get_projects().expect("Couldn't get projects");

    let gpu_2 = Arc::clone(&gpu_helper);

    // Existing Projects
    let project_list = create_rw_signal(projects);
    let loading_project = create_rw_signal(false);

    // New Project
    let show_create_dialog = create_rw_signal(false);
    let new_project_name = create_rw_signal(String::new());

    // Authenication
    let auth_state = create_rw_signal(AuthState {
        token: load_auth_token(),
        is_authenticated: load_auth_token().is_some(),
        subscription: None,
    });

    // Login dialog state
    let show_login_dialog = create_rw_signal(false);
    let email = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let login_error = create_rw_signal(Option::<String>::None);
    let is_logging_in = create_rw_signal(false);

    // create channel
    let (login_tx, login_rx) =
        tokio::sync::mpsc::unbounded_channel::<Result<LoginResponse, String>>();
    let login_result = create_signal_from_tokio_channel(login_rx);

    create_effect(move |_| {
        if let Some(result) = login_result.get() {
            match result {
                Ok(response) => {
                    if let Err(e) = set_authenticated(
                        auth_state,
                        response.jwtData.token,
                        response.jwtData.expiry,
                    ) {
                        login_error.set(Some(format!("Error saving credentials: {}", e)));
                    } else {
                        show_login_dialog.set(false);
                        email.set(String::new());
                        password.set(String::new());
                    }
                }
                Err(e) => {
                    login_error.set(Some(format!("Login failed: {}", e)));
                }
            }
            is_logging_in.set(false);
        }
    });

    // // Create effect to check subscription when authenticated
    // create_effect(move |_| {
    //     if auth_state.get().is_authenticated {
    //         check_subscription(auth_state);
    //     }
    // });

    // Set up subscription checking channel and signal
    let (subscription_tx, subscription_rx) =
        tokio::sync::mpsc::unbounded_channel::<Result<SubscriptionDetails, String>>();
    let subscription_result = create_signal_from_tokio_channel(subscription_rx);

    // Effect to watch authentication and trigger subscription check
    create_effect(move |_| {
        if auth_state.get().is_authenticated && subscription_result.get().is_none() {
            if let Some(token) = auth_state.get().token.as_ref() {
                println!("Spawning...");
                let tx = subscription_tx.clone();
                let token = token.token.clone();

                tokio::spawn(async move {
                    match fetch_subscription_details(&token).await {
                        Ok(subscription) => tx.send(Ok(subscription)),
                        Err(e) => tx.send(Err(e.to_string())),
                    }
                    .expect("Channel send failed");
                });
            }
        }
    });

    // Effect to handle subscription updates
    create_effect(move |_| {
        if auth_state.get().subscription.is_none() {
            if let Some(result) = subscription_result.get() {
                match result {
                    Ok(subscription) => {
                        println!("Setting...");
                        let mut current_state = auth_state.get();
                        current_state.subscription = Some(subscription);
                        auth_state.set(current_state);
                    }
                    Err(e) => {
                        println!("Failed to fetch subscription details: {}", e);
                        // Optionally handle error in UI
                    }
                }
            }
        }
    });

    v_stack((
        dyn_container(
            move || loading_project.get(),
            move |loading_project_real| {
                if (loading_project_real) {
                    alert(
                        floem::common::AlertVariant::Info,
                        "Loading your project...".to_string(),
                    )
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        )
        .into_view(),
        // Subscription status alert
        dyn_container(
            move || auth_state.get(),
            move |state| match state.subscription {
                Some(ref sub)
                    if !matches!(sub.subscription_status.as_str(), "ACTIVE" | "TRIALING") =>
                {
                    alert(
                        floem::common::AlertVariant::Warning,
                        format!(
                            "Your subscription is {}. Consider upgrading!",
                            sub.subscription_status.to_lowercase()
                        ),
                    )
                    .style(|s| s.margin_bottom(16.0))
                    .into_any()
                }
                None if state.is_authenticated => alert(
                    floem::common::AlertVariant::Info,
                    "Checking subscription status...".to_string(),
                )
                .style(|s| s.margin_bottom(16.0))
                .into_any(),
                _ => empty().into_any(),
            },
        ),
        // Authentication status and login button
        h_stack((dyn_container(
            move || auth_state.get().is_authenticated,
            move |is_authenticated| {
                if !is_authenticated {
                    h_stack((
                        alert(
                            floem::common::AlertVariant::Warning,
                            "Please login to create new projects.".to_string(),
                        )
                        .style(|s| s.width(200.0)),
                        button(label(|| "Login"))
                            .on_click(move |_| {
                                show_login_dialog.set(true);
                                EventPropagation::Stop
                            })
                            .style(|s| {
                                s.margin_left(8.0)
                                    .padding(8.0)
                                    .background(Color::rgb(0.0, 122.0, 255.0))
                                    .color(Color::WHITE)
                                    .border_radius(4.0)
                            }),
                    ))
                    .style(|s| s.align_items(AlignItems::Center))
                    .into_any()
                } else {
                    h_stack((
                        label(|| "Logged in"),
                        button(label(|| "Logout"))
                            .on_click(move |_| {
                                let _ = logout(auth_state);
                                EventPropagation::Stop
                            })
                            .style(|s| {
                                s.margin_left(8.0)
                                    .padding(8.0)
                                    .background(Color::rgb(220.0, 53.0, 69.0))
                                    .color(Color::BLACK)
                                    .border_radius(4.0)
                            }),
                    ))
                    .style(|s| s.align_items(AlignItems::Center))
                    .into_any()
                }
            },
        ),))
        .style(|s| s.margin_bottom(16.0)),
        // Project header with create button
        h_stack((
            label(|| "Select a Project").style(|s| s.margin_bottom(4.0)),
            button(label(|| "Create New"))
                .on_click(move |_| {
                    // if auth_state.get().can_create_projects() {
                    show_create_dialog.set(true);
                    // }
                    EventPropagation::Stop
                })
                // .disabled(move || !auth_state.get().can_create_projects())
                .style(move |s| {
                    let can_create = auth_state.get().can_create_projects();
                    s.margin_left(8.0)
                        .padding(8.0)
                        .background(if true {
                            Color::rgb(0.0, 122.0, 255.0)
                        } else {
                            Color::rgb(150.0, 150.0, 150.0)
                        })
                        .color(Color::WHITE)
                        .border_radius(4.0)
                }),
        ))
        .style(|s| s.justify_content(JustifyContent::SpaceBetween)),
        // Create Project Dialog
        dyn_container(
            move || show_create_dialog.get(),
            move |show| {
                if show {
                    v_stack((
                        label(|| "Create New Project"),
                        text_input(new_project_name)
                            .placeholder("Project Name")
                            .style(|s| s.margin_vert(8.0)),
                        h_stack((
                            button(label(|| "Cancel")).on_click(move |_| {
                                show_create_dialog.set(false);
                                new_project_name.set(String::new());
                                EventPropagation::Stop
                            }),
                            button(label(|| "Create"))
                                .on_click(move |_| {
                                    let name = new_project_name.get();
                                    if !name.is_empty() {
                                        create_project_state(name.clone())
                                            .expect("Couldn't create project state");

                                        if let Ok(projects) = get_projects() {
                                            project_list.set(projects);
                                            show_create_dialog.set(false);
                                            new_project_name.set(String::new());
                                        }
                                    }
                                    EventPropagation::Stop
                                })
                                .style(|s| {
                                    s.margin_left(8.0)
                                        .background(Color::rgb(0.0, 122.0, 255.0))
                                        .color(Color::WHITE)
                                }),
                        ))
                        .style(|s| s.justify_content(JustifyContent::FlexEnd)),
                    ))
                    .style(|s| {
                        s.padding(16.0)
                            .background(Color::WHITE)
                            .border_radius(8.0)
                            .border(1.0)
                            .border_color(Color::rgb(200.0, 200.0, 200.0))
                    })
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        // Login Dialog
        dyn_container(
            move || show_login_dialog.get(),
            move |show| {
                if show {
                    v_stack((
                        label(|| "Login").style(|s| s.font_size(18.0).margin_bottom(16.0)),
                        // Error message if any
                        dyn_container(
                            move || login_error.get(),
                            move |error| {
                                if let Some(error_msg) = error {
                                    alert(floem::common::AlertVariant::Error, error_msg)
                                        .style(|s| s.margin_bottom(16.0))
                                        .into_any()
                                } else {
                                    empty().into_any()
                                }
                            },
                        ),
                        text_input(email)
                            .placeholder("Email")
                            .style(|s| s.margin_bottom(8.0)),
                        text_input(password)
                            .placeholder("Password")
                            // .password(true) // TODO: password mask
                            .style(|s| s.margin_bottom(16.0)),
                        h_stack((
                            button(label(|| "Cancel")).on_click(move |_| {
                                show_login_dialog.set(false);
                                email.set(String::new());
                                password.set(String::new());
                                login_error.set(None);
                                EventPropagation::Stop
                            }),
                            button(label(move || {
                                if is_logging_in.get() {
                                    "Logging in..."
                                } else {
                                    "Login"
                                }
                            }))
                            .disabled(move || is_logging_in.get())
                            .on_click({
                                let login_tx = login_tx.clone();

                                move |_| {
                                    let email_val = email.get();
                                    let password_val = password.get();

                                    if email_val.is_empty() || password_val.is_empty() {
                                        login_error
                                            .set(Some("Please fill in all fields".to_string()));
                                        return EventPropagation::Stop;
                                    }

                                    is_logging_in.set(true);
                                    login_error.set(None);

                                    let tx = login_tx.clone();

                                    tokio::spawn(async move {
                                        match login_user(email_val, password_val).await {
                                            Ok(response) => tx.send(Ok(response)),
                                            Err(e) => tx.send(Err(e.to_string())),
                                        }
                                        .expect("Channel send failed");
                                    });

                                    EventPropagation::Stop
                                }
                            })
                            .style(|s| {
                                s.margin_left(8.0)
                                    .background(Color::rgb(0.0, 122.0, 255.0))
                                    .color(Color::WHITE)
                            }),
                        ))
                        .style(|s| s.justify_content(JustifyContent::FlexEnd)),
                    ))
                    .style(|s| {
                        s.padding(16.0)
                            .background(Color::WHITE)
                            .border_radius(8.0)
                            .border(1.0)
                            .border_color(Color::rgb(200.0, 200.0, 200.0))
                            .min_width(300.0)
                    })
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        // Browse Projects
        scroll(
            dyn_stack(
                move || project_list.get(),
                move |project| project.project_id.clone(),
                move |project| {
                    project_item(
                        project.clone(),
                        project_list,
                        project.project_name.clone() + " / " + &project.modified.to_string(),
                        "sphere",
                    )
                    .on_click({
                        let editor = editor.clone();
                        let editor_state = editor_state.clone();
                        // let manager = manager.clone();
                        let gpu_2 = gpu_2.clone();

                        move |_| {
                            if (loading_project.get()) {
                                return EventPropagation::Continue;
                            }

                            loading_project.set(true);

                            // join the WebSocket group for this project
                            // manager.join_group(); // locks and drops the state_helper

                            let mut editor_state = editor_state.lock().unwrap();

                            let uuid = Uuid::from_str(&project.project_id.clone())
                                .expect("Couldn't convert project name to id");

                            let destination_view = "scene".to_string();
                            // no need to set here, the default is scene
                            // let current_view_signal = state_helper
                            //     .current_view_signal
                            //     .expect("Couldn't get current view signal");
                            // current_view_signal.set(destination_view.clone());

                            // retrieve saved state of project and set on helper
                            // restore the saved state to the rendererstate
                            println!("Loading saved state...");
                            let saved_state = load_project_state(uuid.clone().to_string())
                                .expect("Couldn't get Saved State");
                            editor_state.record_state.saved_state = Some(saved_state.clone());

                            // update the UI signal
                            let project_selected = editor_state
                                .project_selected_signal
                                .expect("Couldn't get project selection signal");

                            project_selected.set(uuid.clone());

                            drop(editor_state);

                            let mut editor = editor.lock().unwrap();

                            editor.project_selected = Some(uuid.clone());
                            editor.current_view = destination_view.clone();

                            drop(editor);

                            println!("Project selected {:?}", project.project_name.clone());

                            EventPropagation::Stop
                        }
                    })
                },
            )
            .style(|s| {
                s.width(260.0)
                    .flex()
                    .flex_direction(FlexDirection::Column)
                    .gap(2.0)
            })
            .into_view(),
        ),
    ))
    .style(|s| card_styles(s))
    .style(|s| s.width(300.0))
}

// Function to update authentication state
// pub fn set_authenticated(
//     auth_state: RwSignal<AuthState>,
//     token: String,
//     expiry: Option<chrono::DateTime<chrono::Utc>>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let auth_token = AuthToken { token, expiry };
//     save_auth_token(&auth_token)?;

//     auth_state.set(AuthState {
//         token: Some(auth_token),
//         is_authenticated: true,
//     });

//     Ok(())
// }
pub fn set_authenticated(
    auth_state: RwSignal<AuthState>,
    token: String,
    expiry: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let auth_token = AuthToken { token, expiry };
    save_auth_token(&auth_token)?;

    auth_state.set(AuthState {
        token: Some(auth_token),
        is_authenticated: true,
        subscription: None, // Will be updated by the effect
    });

    Ok(())
}

// Function to handle logout
pub fn logout(auth_state: RwSignal<AuthState>) -> Result<(), Box<dyn std::error::Error>> {
    clear_auth_token()?;

    auth_state.set(AuthState {
        token: None,
        is_authenticated: false,
        subscription: None,
    });

    Ok(())
}

// fn login_user(
//     email: String,
//     password: String,
// ) -> Result<LoginResponse, Box<dyn std::error::Error>> {
//     // use blocking to avoid spawning threads for now
//     let client = reqwest::blocking::Client::builder()
//         .timeout(Duration::from_secs(10))
//         .build()?;

//     let response = client
//         .post("http://localhost:3000/api/auth/login")
//         .json(&LoginRequest { email, password })
//         .send()
//         .expect("Couldn't get login response");

//     if response.status().is_success() {
//         let login_response = response
//             .json::<LoginResponse>()
//             .expect("Couldn't get login json");
//         Ok(login_response)
//     } else {
//         let error_text = response.text().expect("Couldn't get login error text");
//         Err(error_text.into())
//     }
// }

async fn login_user(
    email: String,
    password: String,
) -> Result<LoginResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client
        .post(API_URL.to_owned() + &"/api/auth/login")
        .json(&LoginRequest { email, password })
        .send()
        .await?;

    if response.status().is_success() {
        let login_response = response.json::<LoginResponse>().await?;
        Ok(login_response)
    } else {
        let error_text = response.text().await?;
        Err(error_text.into())
    }
}
