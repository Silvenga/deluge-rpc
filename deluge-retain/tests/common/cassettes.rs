use deluge_rpc::RencodeValue;
use deluge_rpc_mock::{Cassette, Interaction, Request, Response};
use std::collections::BTreeMap;

const GB: i64 = 1_073_741_824;

pub fn login_ok() -> Cassette {
    let mut kwargs = BTreeMap::new();
    kwargs.insert(
        RencodeValue::Str("client_version".into()),
        RencodeValue::Str("deluge-rpc/0.1.0".into()),
    );
    Cassette {
        version: 1,
        recorded_at: "2026-07-04T12:00:00Z".into(),
        daemon_version: Some("2.1.1".into()),
        interactions: vec![Interaction {
            request: Request {
                method: "daemon.login".into(),
                args: RencodeValue::List(vec![
                    RencodeValue::Str("localclient".into()),
                    RencodeValue::Str("secret".into()),
                ]),
                kwargs: RencodeValue::Dict(kwargs),
            },
            response: Response::Ok {
                value: RencodeValue::Int(5),
            },
        }],
    }
}

pub fn login_bad() -> Cassette {
    let mut kwargs = BTreeMap::new();
    kwargs.insert(
        RencodeValue::Str("client_version".into()),
        RencodeValue::Str("deluge-rpc/0.1.0".into()),
    );
    Cassette {
        version: 1,
        recorded_at: "2026-07-04T12:00:00Z".into(),
        daemon_version: Some("2.1.1".into()),
        interactions: vec![Interaction {
            request: Request {
                method: "daemon.login".into(),
                args: RencodeValue::List(vec![
                    RencodeValue::Str("localclient".into()),
                    RencodeValue::Str("secret".into()),
                ]),
                kwargs: RencodeValue::Dict(kwargs),
            },
            response: Response::Error {
                exc_type: "BadLoginError".into(),
                exc_msg: "bad password".into(),
                traceback: String::new(),
            },
        }],
    }
}

pub fn free_space_low() -> Cassette {
    let mut cassette = login_ok();
    cassette.interactions.push(Interaction {
        request: Request {
            method: "core.get_free_space".into(),
            args: RencodeValue::List(vec![RencodeValue::None]),
            kwargs: RencodeValue::List(vec![]),
        },
        response: Response::Ok {
            value: RencodeValue::Int(5 * GB),
        },
    });
    cassette
}

pub fn free_space_high() -> Cassette {
    let mut cassette = login_ok();
    cassette.interactions.push(Interaction {
        request: Request {
            method: "core.get_free_space".into(),
            args: RencodeValue::List(vec![RencodeValue::None]),
            kwargs: RencodeValue::List(vec![]),
        },
        response: Response::Ok {
            value: RencodeValue::Int(30 * GB),
        },
    });
    cassette
}

pub fn torrents_list(info_hash: &str, name: &str, time_added: i64) -> Cassette {
    let mut cassette = free_space_low();

    let keys = [
        "name",
        "state",
        "progress",
        "ratio",
        "total_seeds",
        "num_seeds",
        "time_added",
        "total_done",
        "total_uploaded",
        "is_finished",
        "download_location",
    ];

    let key_values: Vec<RencodeValue> = keys
        .iter()
        .map(|k| RencodeValue::Str(k.to_string()))
        .collect();

    let mut fields = BTreeMap::new();
    fields.insert(
        RencodeValue::Str("name".into()),
        RencodeValue::Str(name.into()),
    );
    fields.insert(
        RencodeValue::Str("state".into()),
        RencodeValue::Str("Seeding".into()),
    );
    fields.insert(
        RencodeValue::Str("progress".into()),
        RencodeValue::Float(100.0),
    );
    fields.insert(RencodeValue::Str("ratio".into()), RencodeValue::Float(3.0));
    fields.insert(
        RencodeValue::Str("total_seeds".into()),
        RencodeValue::Int(50),
    );
    fields.insert(RencodeValue::Str("num_seeds".into()), RencodeValue::Int(5));
    fields.insert(
        RencodeValue::Str("time_added".into()),
        RencodeValue::Int(time_added),
    );
    fields.insert(
        RencodeValue::Str("total_done".into()),
        RencodeValue::Int(2 * GB),
    );
    fields.insert(
        RencodeValue::Str("total_uploaded".into()),
        RencodeValue::Int(0),
    );
    fields.insert(
        RencodeValue::Str("is_finished".into()),
        RencodeValue::Bool(true),
    );
    fields.insert(
        RencodeValue::Str("download_location".into()),
        RencodeValue::Str("/data".into()),
    );

    let mut torrent_dict = BTreeMap::new();
    torrent_dict.insert(
        RencodeValue::Str(info_hash.into()),
        RencodeValue::Dict(fields),
    );

    let mut kwargs = BTreeMap::new();
    kwargs.insert(RencodeValue::Str("diff".into()), RencodeValue::Bool(false));

    cassette.interactions.push(Interaction {
        request: Request {
            method: "core.get_torrents_status".into(),
            args: RencodeValue::List(vec![
                RencodeValue::Dict(BTreeMap::new()),
                RencodeValue::List(key_values),
            ]),
            kwargs: RencodeValue::Dict(kwargs),
        },
        response: Response::Ok {
            value: RencodeValue::Dict(torrent_dict),
        },
    });
    cassette
}

pub fn remove_torrent(info_hash: &str, time_added: i64) -> Cassette {
    let mut cassette = torrents_list(info_hash, "old-torrent", time_added);
    cassette.interactions.push(Interaction {
        request: Request {
            method: "core.remove_torrent".into(),
            args: RencodeValue::List(vec![
                RencodeValue::Str(info_hash.into()),
                RencodeValue::Bool(true),
            ]),
            kwargs: RencodeValue::List(vec![]),
        },
        response: Response::Ok {
            value: RencodeValue::Bool(true),
        },
    });
    cassette
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_login_bad_cassette_then_has_error_response() {
        let cassette = login_bad();
        assert_eq!(cassette.interactions.len(), 1);
        assert_eq!(cassette.interactions[0].request.method, "daemon.login");
        assert!(matches!(
            cassette.interactions[0].response,
            deluge_rpc_mock::Response::Error { .. }
        ));
    }
}
