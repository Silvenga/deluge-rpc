use serde::{Deserialize, Serialize};

/// Configuration for the Scheduler plugin.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchedulerConfig {
    /// Download speed limit in "Yellow" state (KiB/s); -1.0 = unlimited.
    pub low_down: f64,
    /// Upload speed limit in "Yellow" state (KiB/s); -1.0 = unlimited.
    pub low_up: f64,
    /// Max active torrents in "Yellow" state; -1 = unlimited.
    pub low_active: i64,
    /// Max active downloads in "Yellow" state; -1 = unlimited.
    pub low_active_down: i64,
    /// Max active seeds in "Yellow" state; -1 = unlimited.
    pub low_active_up: i64,
    /// Schedule grid indexed as `[hour][weekday]`.
    /// Values: 0=Green, 1=Yellow, 2=Red.
    pub button_state: Vec<Vec<i64>>,
}

/// Current schedule state returned by `scheduler.get_state`.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum SchedulerState {
    /// Unlimited bandwidth.
    #[serde(rename = "Green")]
    Green,
    /// Limited bandwidth.
    #[serde(rename = "Yellow")]
    Yellow,
    /// Stopped.
    #[serde(rename = "Red")]
    Red,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RencodeValue;
    use std::collections::BTreeMap;

    fn make_dict(entries: Vec<(&str, RencodeValue)>) -> RencodeValue {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(RencodeValue::Str(k.into()), v);
        }
        RencodeValue::Dict(map)
    }

    fn make_24x7_grid() -> Vec<Vec<i64>> {
        (0..24).map(|_| vec![0i64, 0, 0, 0, 0, 0, 0]).collect()
    }

    #[test]
    fn when_scheduler_config_dict_then_fields_populate() {
        let grid = make_24x7_grid();
        let grid_value = RencodeValue::List(
            grid.iter()
                .map(|row| RencodeValue::List(row.iter().map(|&v| RencodeValue::Int(v)).collect()))
                .collect(),
        );

        let value = make_dict(vec![
            ("low_down", RencodeValue::Float(100.0)),
            ("low_up", RencodeValue::Float(50.0)),
            ("low_active", RencodeValue::Int(5)),
            ("low_active_down", RencodeValue::Int(3)),
            ("low_active_up", RencodeValue::Int(2)),
            ("button_state", grid_value),
        ]);

        let result: SchedulerConfig = SchedulerConfig::deserialize(&value).expect("deserialize");

        assert!((result.low_down - 100.0).abs() < f64::EPSILON);
        assert!((result.low_up - 50.0).abs() < f64::EPSILON);
        assert_eq!(result.low_active, 5);
        assert_eq!(result.low_active_down, 3);
        assert_eq!(result.low_active_up, 2);
        assert_eq!(result.button_state.len(), 24);
        assert_eq!(result.button_state[0].len(), 7);
    }

    #[test]
    fn when_scheduler_state_green_then_deserializes() {
        let value = RencodeValue::Str("Green".into());

        let result: SchedulerState = SchedulerState::deserialize(&value).expect("deserialize");

        assert_eq!(result, SchedulerState::Green);
    }

    #[test]
    fn when_scheduler_state_yellow_then_deserializes() {
        let value = RencodeValue::Str("Yellow".into());

        let result: SchedulerState = SchedulerState::deserialize(&value).expect("deserialize");

        assert_eq!(result, SchedulerState::Yellow);
    }

    #[test]
    fn when_scheduler_state_red_then_deserializes() {
        let value = RencodeValue::Str("Red".into());

        let result: SchedulerState = SchedulerState::deserialize(&value).expect("deserialize");

        assert_eq!(result, SchedulerState::Red);
    }
}
