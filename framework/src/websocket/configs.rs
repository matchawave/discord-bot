use colored::Colorize;
use tokio_tungstenite::{
    Connector,
    tungstenite::{ClientRequestBuilder, http::Uri, protocol::WebSocketConfig},
};
use utils::error;

/// Read buffer capacity. This buffer is eagerly allocated and used for receiving messages.
///
/// For high read load scenarios a larger buffer, e.g. 128 KiB, improves performance.
///
/// For scenarios where you expect a lot of connections and don't need high read load
/// performance a smaller buffer, e.g. 4 KiB, would be appropriate to lower total
/// memory usage.
///
/// The default value is 128 KiB.
const READ_BUFFER_SIZE: usize = 128 * 1024;

/// The target minimum size of the write buffer to reach before writing the data to the underlying stream.
/// The default value is 128 KiB.
///
/// If set to `0` each message will be eagerly written to the underlying stream.
/// It is often more optimal to allow them to buffer a little, hence the default value.
///
/// Note: [`flush`](WebSocket::flush) will always fully write the buffer regardless.
const WRITE_BUFFER_SIZE: usize = 128 * 1024;

/// The max size of the write buffer in bytes. Setting this can provide backpressure
/// in the case the write buffer is filling up due to write errors.
/// The default value is unlimited.
///
/// Note: The write buffer only builds up past [`write_buffer_size`](Self::write_buffer_size)
/// when writes to the underlying stream are failing. So the **write buffer can not
/// fill up if you are not observing write errors even if not flushing**.
///
/// Note: Should always be at least [`write_buffer_size + 1 message`](Self::write_buffer_size)
/// and probably a little more depending on error handling strategy.
const MAX_WRITE_BUFFER_SIZE: usize = usize::MAX;

/// The maximum size of an incoming message. `None` means no size limit. The default value is 64 MiB
/// which should be reasonably big for all normal use-cases but small enough to prevent
/// memory eating by a malicious user.
const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;

/// The maximum size of a single incoming message frame. `None` means no size limit. The limit is for
/// frame payload NOT including the frame header. The default value is 16 MiB which should
/// be reasonably big for all normal use-cases but small enough to prevent memory eating
/// by a malicious user.
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

/// When set to `true`, the server will accept and handle unmasked frames
/// from the client. According to the RFC 6455, the server must close the
/// connection to the client in such cases, however it seems like there are
/// some popular libraries that are sending unmasked frames, ignoring the RFC.
/// By default this option is set to `false`, i.e. according to RFC 6455.
const ACCEPT_UNMASKED_FRAMES: bool = false;

pub(super) fn create_request(ws_url: String, token: &str) -> ClientRequestBuilder {
    let ws_text = "websocket".yellow();

    let uri = match Uri::try_from(&ws_url) {
        Ok(u) => u,
        Err(e) => {
            error!(
                "({ws_text}) Invalid WebSocket URI: {:?}, error: {:?}",
                ws_url, e
            );
            panic!("Invalid WebSocket URI");
        }
    };
    ClientRequestBuilder::new(uri).with_header("client", format!("DiscordBot {}", token))
}

pub(super) fn create_config() -> WebSocketConfig {
    WebSocketConfig::default()
        .max_message_size(Some(MAX_MESSAGE_SIZE))
        .max_frame_size(Some(MAX_FRAME_SIZE))
        .accept_unmasked_frames(ACCEPT_UNMASKED_FRAMES)
        .read_buffer_size(READ_BUFFER_SIZE)
        .write_buffer_size(WRITE_BUFFER_SIZE)
        .max_write_buffer_size(MAX_WRITE_BUFFER_SIZE)
}

pub(super) fn create_connector() -> Connector {
    Connector::Plain
}
