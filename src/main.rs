use tokio::time::{sleep, Duration};
use std::sync::{Arc as Arc_1f4a602b, Mutex as Mutex_1f4a602b};
use std::sync::mpsc as std_mpsc_1f4a602b;
use std::thread::{spawn as spawn_1f4a602b};
use std::fs::File as File_1f4a602b;
use std::io::{BufWriter as BufWriter_1f4a602b};
use hound::{self as hound_1f4a602b, WavWriter as WavWriter_1f4a602b, SampleFormat as SampleFormat_1f4a602b, WavSpec as WavSpec_1f4a602b, Error as HoundError_1f4a602b};
use druid::{AppLauncher, Command as druidCommand, DelegateCtx, Env, Handled, Lens, Target, Widget, WidgetExt, WindowDesc};
use druid::widget::{Button, Flex};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use anyhow::{Result}; 
use std::fs::File as stdFile;
use async_std::task;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use druid::{Data,Selector,AppDelegate};
use warp::Filter;

//
// Struct for GUI
//
#[derive(Clone, Data, Lens)]
struct AppState {
    input_text: String,
    received_text: String,
    received_text_visible: bool,
    chat_id: String,
}

struct Delegate;


//

// 
// API web server funcs
// 

// Handler function for /start_recording
// async fn handle_start_recording() -> Result<impl warp::Reply, warp::Rejection> {

//     Ok(warp::reply::with_status("Recording started in background", warp::http::StatusCode::OK))
// }
// Handler function for /start_recording
// async fn handle_stop_recording() -> Result<impl warp::Reply, warp::Rejection> {
//     // Start a new task for start_recording
    

//     Ok(warp::reply::with_status("Recording stop in background", warp::http::StatusCode::OK))
// }



// Custom command to update the application state

pub const BUILD_MESSAGE_GPT: Selector<String> = Selector::new("build_message_gpt");
pub const MY_CUSTOM_COMMAND: Selector<String> = Selector::new("my_custom_command");

// Dummy example of conversion logic


impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &druidCommand,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if let Some(gpt_result) = cmd.get(MY_CUSTOM_COMMAND) {
            // Now, `gpt_result` is a `&String` containing your GPT result
            // Update your application state or perform other actions as needed
            data.received_text = gpt_result.clone();
            

            Handled::Yes
        } else {
            Handled::No
        }
    }
}

// Global stop signal
// lazy_static::lazy_static! {
//     static ref STOP_SIGNAL: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>> = Arc::new(Mutex::new(None));
// }

lazy_static::lazy_static! {
    static ref STOP_SIGNAL: Arc<Mutex<Option<tokio::sync::watch::Sender<bool>>>> = Arc::new(Mutex::new(None));
}


// Function to start recording audio
async fn async_capture_voice(chat_id : i32) {
    println!("Started Stopping recording");
    let (tx, rx) = std_mpsc_1f4a602b::channel::<bool>();
    // let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let (stop_tx,mut stop_rx) = tokio::sync::watch::channel(false);
    println!("stop_tx value: {:?}", stop_tx);
    *STOP_SIGNAL.lock().unwrap() = Some(stop_tx);
    let recording = Arc_1f4a602b::new(Mutex_1f4a602b::new(false));
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    let config = device.default_input_config().unwrap();
    println!("Default input config: {:?}", config);

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    let spec = WavSpec_1f4a602b {
        channels: channels,
        sample_rate: sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat_1f4a602b::Int,
    };

    // Use File::create here with the correct method usage
    let file = File_1f4a602b::create("recording.wav").expect("Failed to create file");
    let writer = BufWriter_1f4a602b::new(file);
    let wav_writer = WavWriter_1f4a602b::new(writer, spec).expect("Failed to create WAV writer");
    let shared_writer = Arc_1f4a602b::new(Mutex_1f4a602b::new(Some(wav_writer)));

    let recording_clone = recording.clone();
    let writer_clone = shared_writer.clone();
    
    
    // spawn_1f4a602b(move || {
    //     start_recording(recording_clone, tx, writer_clone, chat_id ,0);
    // });
    tokio::spawn(async move {
        start_recording(recording_clone, tx, writer_clone, chat_id, 0);
    });

    
    // Record 10s
    // stop_rx.await.ok();
    while !*stop_rx.borrow() {
        stop_rx.changed().await.unwrap();
    }
    // sleep(Duration::from_secs(10)).await;

    println!("Stopping recording");
    stop_recording(recording, shared_writer, chat_id , 0, "NULL").await;

    // Abort the server after recording


}


fn start_recording(recording: Arc_1f4a602b<Mutex_1f4a602b<bool>>, tx: std::sync::mpsc::Sender<bool>, writer: Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>>,
    chat_id: i32,                // Add chat_id parameter
    voice_recording_id: i32      // Add voice_recording_id parameter
) {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    println!("Using input device: {}", device.name().unwrap_or("<unknown>".to_string()));

    let config = device.default_input_config().unwrap();
    println!("Default input config: {:?}", config);

    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;

    let writer_clone = writer.clone();
    let err_fn = |err| eprintln!("An error occurred on the stream: {}", err);
    let stream = match device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if let Some(writer) = writer_clone.lock().unwrap().as_mut() {
                for &sample in data {
                    let amplitude = (sample * i16::MAX as f32) as i16;
                    if let Err(e) = writer.write_sample(amplitude) {
                        eprintln!("Failed to write sample: {}", e);
                        break;
                    }
                }
            }
        },
        err_fn,
        None,
    ) {
        Ok(stream) => stream,
        Err(e) => {
            let _ = print!("Failed to build input stream: {}", e);
            return;
        }
    };

    stream.play().unwrap();
    println!("Recording...");

    *recording.lock().unwrap() = true;

    tx.send(true).expect("Failed to send start signal");

    // Continue recording until a stop signal is received
    while *recording.lock().unwrap() {}

    println!("Recording thread exiting...");
}


async fn stop_recording(
    recording: Arc<Mutex<bool>>, 
    writer: Arc_1f4a602b<Mutex_1f4a602b<Option<WavWriter_1f4a602b<BufWriter_1f4a602b<File_1f4a602b>>>>>,
    chat_id: i32,
    voice_recording_id: i32,
    content: &str) {
    let mut rec = recording.lock().unwrap();
    *rec = false;

    finalize_wav_file(writer.clone()).expect("Failed to finalize WAV file");
    println!("Recording stopped and file finalized.");
    }

fn finalize_wav_file(writer: Arc_1f4a602b<Mutex_1f4a602b<Option<WavWriter_1f4a602b<BufWriter_1f4a602b<File_1f4a602b>>>>>) -> Result<(), hound_1f4a602b::Error> {
    let maybe_writer = {
        let mut writer_lock = writer.lock().unwrap();
        writer_lock.take()
    };

    if let Some(mut writer) = maybe_writer {
        writer.finalize()
    } else {
        Ok(())
    }
}


// HTTP Handlers
async fn handle_start_recording() -> Result<impl warp::Reply, warp::Rejection> {
    tokio::spawn(async {
        async_capture_voice(1).await;
    });
    Ok(warp::reply::with_status("Recording started in background", warp::http::StatusCode::OK))
}

async fn handle_stop_recording() -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(stop_tx) = STOP_SIGNAL.lock().unwrap().take() {
        let _ = stop_tx.send(true);
    }
    // Send a stop signal to the recording task
    // This is a placeholder; you need to implement the actual stop mechanism
    Ok(warp::reply::with_status("Recording stopped in background", warp::http::StatusCode::OK))
}



// UI Components
// 



fn build_ui_combined() -> impl Widget<AppState> {
    // TextBox for updates
        let start_rec = Button::new("Start").on_click(|_ctx, data: &mut AppState, _env| {
            // Parse chat_id from the AppState instance
            let chat_id_int = 1;
            // task::block_on(async_capture_voice(chat_id_int));
            tokio::spawn(async_capture_voice(chat_id_int));
        });

        let stop_rec = Button::new("Stop").on_click(|_ctx, data: &mut AppState, _env| {
            // Parse chat_id from the AppState instance
            let chat_id_int = 1;
            if let Some(stop_tx) = STOP_SIGNAL.lock().unwrap().take() {
                let _ = stop_tx.send(true);
            }
        });

    // Layout with elements
    Flex::column()
        .with_spacer(50.0)
        .with_child(
            Flex::row()
                .with_child(start_rec)
                .with_child(stop_rec)
        )
}


//
// Main func
//
#[tokio::main]
async fn main() {
    // let (tx, mut rx) = mpsc::channel(100);
    let main_window = WindowDesc::new(build_ui_combined())
    .title("Voice Recording - Rust Interview Task")
    .window_size((200.0, 200.0));

    let launcher = AppLauncher::with_window(main_window);
    
    
    // Launch the GUI
    let initial_state = AppState {
        input_text: "".to_string(),
        received_text: "".to_string(),
        received_text_visible: true,
        chat_id: "1".to_string(),
    };

    // Launch the web server
    let routes = warp::path("start_recording")
        .and(warp::get())
        .and_then(handle_start_recording)
        .or(warp::path("stop_recording")
            .and(warp::get())
            .and_then(handle_stop_recording));

    tokio::spawn(async move {
        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    });

    launcher
    .delegate(Delegate {})
    .launch(initial_state)
    .expect("Failed to launch application");
}
