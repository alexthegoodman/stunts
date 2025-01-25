use std::sync::{Arc, Mutex};

use chrono::Local;
use floem::event::EventPropagation;
use floem::ext_event::create_signal_from_tokio_channel;
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalGet, SignalUpdate};
use floem::views::Decorators;
use floem::{
    event::EventListener,
    reactive::create_signal,
    style::Style,
    views::{button, h_stack, label, v_stack},
    View,
};
use stunts_engine::animations::Sequence;
use stunts_engine::editor::{Viewport, WindowSize};
use stunts_engine::export::exporter::{ExportProgress, Exporter};
use stunts_engine::timelines::SavedTimelineStateConfig;
use tokio::sync::mpsc;

use crate::editor_state::EditorState;

use std::thread;

// Messages we'll send to the export thread
pub enum ExportCommand {
    StartExport {
        output_path: String,
        window_size: WindowSize,
        sequences: Vec<Sequence>,
        saved_timeline_state_config: SavedTimelineStateConfig,
        video_width: u32,
        video_height: u32,
        total_duration_s: f64,
        progress_tx: mpsc::UnboundedSender<ExportProgress>,
    },
    Stop,
}

// Create a dedicated thread for handling the non-Send encoder
pub fn spawn_export_thread() -> mpsc::Sender<ExportCommand> {
    println!("Spawning export thread...");

    let (cmd_tx, mut cmd_rx) = mpsc::channel(1);

    thread::spawn(move || {
        // Create event loop for the thread
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    ExportCommand::StartExport {
                        output_path,
                        window_size,
                        sequences,
                        saved_timeline_state_config,
                        video_width,
                        video_height,
                        total_duration_s,
                        progress_tx,
                    } => {
                        println!("ExportCommand::StartExport received...");

                        // Create Exporter in this thread where it will be used
                        let mut exporter = Exporter::new(&output_path);

                        println!("Exporter running...");

                        match exporter
                            .run(
                                window_size,
                                sequences,
                                saved_timeline_state_config,
                                video_width,
                                video_height,
                                total_duration_s,
                                progress_tx.clone(),
                            )
                            .await
                        {
                            Ok(_) => {
                                let _ = progress_tx.send(ExportProgress::Complete);
                            }
                            Err(e) => {
                                let _ = progress_tx.send(ExportProgress::Error(e.to_string()));
                            }
                        }
                    }
                    ExportCommand::Stop => break,
                }
            }
        });
    });

    cmd_tx
}

// pub fn export_widget(
//     editor_state: Arc<Mutex<EditorState>>,
//     viewport: Arc<Mutex<Viewport>>,
//     sequence_timeline_signal: RwSignal<TimelineState>,
// ) -> impl View {
//     let (progress_tx, progress_rx) = mpsc::unbounded_channel();
//     let progress = create_signal_from_tokio_channel(progress_rx);
//     let is_exporting = create_rw_signal(false);
//     let progress_text = create_rw_signal(String::from("Ready to export"));

//     // Create the export thread and keep its sender
//     let export_thread_tx = create_rw_signal(spawn_export_thread());

//     create_effect(move |_| {
//         if let Some(result) = progress.get() {
//             match result {
//                 ExportProgress::Progress(percent) => {
//                     progress_text.set(format!("Exporting: {:.1}%", percent));
//                 }
//                 ExportProgress::Complete => {
//                     progress_text.set("Export complete!".to_string());
//                     is_exporting.set(false);
//                 }
//                 ExportProgress::Error(err) => {
//                     progress_text.set(format!("Export failed: {}", err));
//                     is_exporting.set(false);
//                 }
//             }
//         }
//     });

//     v_stack((h_stack((
//         button(label(move || "Export Video"))
//             .style(|s| s.background(Color::rgb8(255, 25, 25)).color(Color::WHITE))
//             .on_click(move |_| {
//                 if is_exporting.get() {
//                     return EventPropagation::Stop;
//                 }

//                 is_exporting.set(true);
//                 let progress_tx = progress_tx.clone();

//                 progress_text.set("Starting export...".to_string());

//                 println!("Starting export...");

//                 let mut editor_state = editor_state.lock().unwrap();
//                 let viewport = viewport.lock().unwrap();

//                 let window_size = WindowSize {
//                     width: viewport.width as u32,
//                     height: viewport.height as u32,
//                 };
//                 let mut new_state = editor_state
//                     .saved_state
//                     .as_mut()
//                     .expect("Couldn't get Saved State")
//                     .clone();

//                 let sequences = new_state.sequences.clone();

//                 let timeline_state = sequence_timeline_signal.get();
//                 let saved_timeline_state_config = timeline_state.to_config();
//                 let total_duration_s: f64 = saved_timeline_state_config
//                     .timeline_sequences
//                     .iter()
//                     .map(|s| s.duration_ms as f64 / 1000.0)
//                     .sum();

//                 println!("Sending tx export command...");

//                 // Send command to export thread
//                 // TODO: send_result is a future, how to solve? Exporter must be done in raw thread, not tokio thread, as only raw thread is compatable with wgpu, Media Foundation, etc
//                 let export_thread = export_thread_tx.get();
//                 let send_result = export_thread.send(ExportCommand::StartExport {
//                     output_path: "D:/projects/common/stunts/output.mp4".to_string(),
//                     window_size,
//                     sequences: sequences.clone(),
//                     saved_timeline_state_config: saved_timeline_state_config.clone(),
//                     video_width: 1920,
//                     video_height: 1080,
//                     total_duration_s,
//                     progress_tx,
//                 });

//                 println!("Tx command sent...");

//                 EventPropagation::Stop
//             })
//             .disabled(move || is_exporting.get()),
//         label(move || progress_text.get()),
//     ))
//     .style(|s| s.gap(10.0)),))
//     .style(|s| s.padding(10.0))
// }

pub fn export_widget(
    editor_state: Arc<Mutex<EditorState>>,
    viewport: Arc<Mutex<Viewport>>,
    // sequence_timeline_signal: RwSignal<TimelineState>,
    sequence_timeline: RwSignal<Option<SavedTimelineStateConfig>>,
) -> impl View {
    let (progress_tx, progress_rx) = mpsc::unbounded_channel();
    let progress = create_signal_from_tokio_channel(progress_rx);
    let is_exporting = create_rw_signal(false);
    let progress_text = create_rw_signal(String::from("Ready to export"));

    // Create the export thread and keep its sender
    let export_thread_tx = create_rw_signal(spawn_export_thread());

    create_effect(move |_| {
        if let Some(result) = progress.get() {
            match result {
                ExportProgress::Progress(percent) => {
                    progress_text.set(format!("Exporting: {:.1}%", percent));
                }
                ExportProgress::Complete => {
                    progress_text.set("Export complete!".to_string());
                    is_exporting.set(false);
                }
                ExportProgress::Error(err) => {
                    progress_text.set(format!("Export failed: {}", err));
                    is_exporting.set(false);
                }
            }
        }
    });

    v_stack((h_stack((
        button(label(move || "Export Video"))
            .style(|s| s.background(Color::rgb8(255, 25, 25)).color(Color::WHITE))
            .on_click(move |_| {
                if is_exporting.get() {
                    return EventPropagation::Stop;
                }

                is_exporting.set(true);
                let progress_tx = progress_tx.clone();
                progress_text.set("Starting export...".to_string());

                println!("Starting export...");

                // Clone necessary values before spawning thread
                let editor_state = editor_state.clone();
                let viewport = viewport.clone();
                // let sequence_timeline_signal = sequence_timeline_signal;
                let export_thread_tx = export_thread_tx.get();

                let editor_state = editor_state.lock().unwrap();
                let viewport = viewport.lock().unwrap();

                let window_size = WindowSize {
                    width: viewport.width as u32,
                    height: viewport.height as u32,
                };
                let mut new_state = editor_state
                    .record_state
                    .saved_state
                    .as_ref()
                    .expect("Couldn't get Saved State")
                    .clone();

                let sequences = new_state.sequences.clone();

                let saved_timeline_state_config =
                    sequence_timeline.get().expect("Couldn't get a timeline");

                // get total duration of sequences that are in both the timleine and the saved state
                let matching_sequences = sequences
                    .iter()
                    .filter(|s| {
                        saved_timeline_state_config
                            .timeline_sequences
                            .iter()
                            .any(|ts| ts.id == s.id)
                    })
                    .collect::<Vec<&Sequence>>();

                // let total_duration_s: f64 = saved_timeline_state_config
                //     .clone()
                //     .timeline_sequences
                //     .iter()
                //     .map(|s| s.duration_ms as f64 / 1000.0)
                //     .sum();

                let total_duration_s = matching_sequences
                    .iter()
                    .map(|s| s.duration_ms as f64 / 1000.0)
                    .sum();

                let filename = format!("export_{}.mp4", Local::now().format("%Y-%m-%d_%H-%M-%S"));

                // Spawn a new thread to handle the setup and sending
                thread::spawn(move || {
                    println!("Sending tx export command...");

                    // Create a new runtime for this thread to handle the async send
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        match export_thread_tx
                            .send(ExportCommand::StartExport {
                                output_path: "D:/projects/common/stunts/".to_string() + &filename,
                                window_size,
                                sequences,
                                saved_timeline_state_config: saved_timeline_state_config.clone(),
                                video_width: 1920,
                                video_height: 1080,
                                total_duration_s,
                                progress_tx: progress_tx.clone(),
                            })
                            .await
                        {
                            Ok(_) => println!("Export command sent successfully"),
                            Err(e) => {
                                println!("Failed to send export command: {}", e);
                                // You might want to send this error through the progress channel
                                let _ = progress_tx.send(ExportProgress::Error(
                                    "Failed to start export".to_string(),
                                ));
                            }
                        }
                    });
                });

                EventPropagation::Stop
            })
            .disabled(move || is_exporting.get()),
        label(move || progress_text.get()),
    ))
    .style(|s| s.gap(10.0)),))
    .style(|s| s.padding(10.0))
}
