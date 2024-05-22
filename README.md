# Voice recording - Rust Technical Task

There are several unused variables in this Rust application, please do not touch any unused vars eg. voice_recording_id, chat_id etc.

1. Change the start_recording to be async so the GUI running on the main thread is not blocked, and so remove the sleep here `sleep(Duration::from_secs(10)).await;`
2. Expose start_recording and stop_recording on HTTP GET routes so they can be called outside of the application, ensuring the webserver is launched async. Placeholder handlers have been included `handle_start_recording` and `handle_stop_recording`
3. Test your work to ensure you can create a recording.wav using the below commands
`curl localhost:3030/start_recording`
`curl localhost:3030/stop_recording`


