use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTrackArgs {
    pub input_path: String,
    pub start: f64,
    pub end: f64,
    pub target_box: TargetBox,
    pub sample_interval: f64,
}

#[derive(Deserialize)]
pub struct TargetBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize)]
pub struct AutoTrackKeyframe {
    pub time: f64,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

#[derive(Serialize)]
pub struct AutoTrackResult {
    pub keyframes: Vec<AutoTrackKeyframe>,
}

#[tauri::command]
pub async fn auto_track(_args: AutoTrackArgs) -> Result<AutoTrackResult, String> {
    Err("Auto tracking is not yet supported on this build. Use Manual or Record Path tracking instead.".to_string())
}
