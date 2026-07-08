use crate::server::matcher::Matcher;
use crate::server::read_frame::{ReadFrameError, read_frame};
use crate::server::write_frame::write_frame;
use crate::{Interaction, InteractionResponse};
use deluge_rpc_client::RencodeValue;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

const LOGIN_METHOD: &str = "daemon.login";
const AUTH_LEVEL_ADMIN: i64 = 5;
const RPC_RESPONSE: i64 = 1;
const RPC_ERROR: i64 = 2;

pub async fn handle_connection(mut tls: TlsStream<TcpStream>, matcher: &Matcher) {
    loop {
        let raw = match read_frame(&mut tls).await {
            Ok(bytes) => bytes,
            Err(ReadFrameError::Eof) => return,
            Err(ReadFrameError::Io(e)) => {
                tracing::debug!(error = %e, "read frame error, closing connection");
                return;
            }
        };
        let decoded = match RencodeValue::decode(&raw) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let Some((id, method, args)) = extract_request(&decoded) else {
            continue;
        };
        let response = match matcher.find_match(&method, &args) {
            Some(interaction) => build_response_from_interaction(id, &interaction),
            None if method == LOGIN_METHOD => build_login_response(id),
            None => build_unknown_method_response(id, &method),
        };
        let encoded = response.encode();
        if write_frame(&mut tls, &encoded).await.is_err() {
            return;
        }
    }
}

fn extract_request(decoded: &RencodeValue) -> Option<(u32, String, RencodeValue)> {
    let outer = match decoded {
        RencodeValue::List(items) if items.len() == 1 => &items[0],
        _ => return None,
    };
    let inner = match outer {
        RencodeValue::List(parts) if parts.len() >= 2 => parts,
        _ => return None,
    };
    let id = match &inner[0] {
        RencodeValue::Int(i) => u32::try_from(*i).ok()?,
        _ => return None,
    };
    let method = match &inner[1] {
        RencodeValue::Str(s) => s.clone(),
        _ => return None,
    };
    let args = inner.get(2).cloned().unwrap_or(RencodeValue::List(vec![]));
    Some((id, method, args))
}

fn build_response_from_interaction(id: u32, interaction: &Interaction) -> RencodeValue {
    let inner = match &interaction.response {
        InteractionResponse::Ok { value } => vec![
            RencodeValue::Int(RPC_RESPONSE),
            RencodeValue::Int(i64::from(id)),
            value.clone(),
        ],
        InteractionResponse::Error {
            exc_type,
            exc_msg,
            traceback,
        } => vec![
            RencodeValue::Int(RPC_ERROR),
            RencodeValue::Int(i64::from(id)),
            RencodeValue::Str(exc_type.clone()),
            RencodeValue::Str(exc_msg.clone()),
            RencodeValue::Str(traceback.clone()),
        ],
    };
    RencodeValue::List(inner)
}

fn build_login_response(id: u32) -> RencodeValue {
    let inner = vec![
        RencodeValue::Int(RPC_RESPONSE),
        RencodeValue::Int(i64::from(id)),
        RencodeValue::Int(AUTH_LEVEL_ADMIN),
    ];
    RencodeValue::List(inner)
}

fn build_unknown_method_response(id: u32, method: &str) -> RencodeValue {
    let inner = vec![
        RencodeValue::Int(RPC_ERROR),
        RencodeValue::Int(i64::from(id)),
        RencodeValue::Str("UnknownMethod".into()),
        RencodeValue::Str(format!("no cassette entry for method '{method}'")),
        RencodeValue::Str(String::new()),
    ];
    RencodeValue::List(inner)
}
