use serde::Deserialize;

/// A command registered with the Execute plugin.
///
/// Deserialized from a tuple `(command_id, event, command)`.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExecuteCommand {
    /// SHA1 hex string identifying the command.
    pub command_id: String,
    /// The torrent event that triggers this command.
    pub event: ExecuteEvent,
    /// The shell command string to execute.
    pub command: String,
}

/// Torrent events that can trigger an Execute command.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ExecuteEvent {
    /// Triggered when a torrent completes downloading.
    #[serde(rename = "complete")]
    Complete,
    /// Triggered when a torrent is added.
    #[serde(rename = "added")]
    Added,
    /// Triggered when a torrent is removed.
    #[serde(rename = "removed")]
    Removed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;

    #[test]
    fn when_execute_command_tuple_then_fields_populate() {
        let value = RencodeValue::List(vec![
            RencodeValue::Str("abc123def456".into()),
            RencodeValue::Str("complete".into()),
            RencodeValue::Str("echo done".into()),
        ]);

        let result: ExecuteCommand = ExecuteCommand::deserialize(&value).expect("deserialize");

        assert_eq!(
            result,
            ExecuteCommand {
                command_id: "abc123def456".into(),
                event: ExecuteEvent::Complete,
                command: "echo done".into(),
            }
        );
    }

    #[test]
    fn when_execute_event_added_then_deserializes() {
        let value = RencodeValue::Str("added".into());

        let result: ExecuteEvent = ExecuteEvent::deserialize(&value).expect("deserialize");

        assert_eq!(result, ExecuteEvent::Added);
    }

    #[test]
    fn when_execute_event_removed_then_deserializes() {
        let value = RencodeValue::Str("removed".into());

        let result: ExecuteEvent = ExecuteEvent::deserialize(&value).expect("deserialize");

        assert_eq!(result, ExecuteEvent::Removed);
    }

    #[test]
    fn when_execute_event_complete_then_deserializes() {
        let value = RencodeValue::Str("complete".into());

        let result: ExecuteEvent = ExecuteEvent::deserialize(&value).expect("deserialize");

        assert_eq!(result, ExecuteEvent::Complete);
    }
}
