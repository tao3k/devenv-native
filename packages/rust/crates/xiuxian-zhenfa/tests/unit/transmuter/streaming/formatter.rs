use super::{AnsiFormatter, DisplayStyle, ZhenfaStreamingEvent};

#[test]
fn ansi_formatter_formats_plain_and_ansi_output() {
    let event = ZhenfaStreamingEvent::Status(std::sync::Arc::from("ready"));

    let plain = AnsiFormatter::new().format(&event);
    assert!(plain.contains("Status"));

    let ansi = AnsiFormatter::with_style(DisplayStyle::Ansi).format(&event);
    assert!(ansi.contains("\x1b[36m"));
}

#[test]
fn ansi_formatter_formats_json_output() {
    let event = ZhenfaStreamingEvent::Status(std::sync::Arc::from("ready"));
    let json = AnsiFormatter::with_style(DisplayStyle::Json).format(&event);

    assert!(json.contains("\"type\":\"Status\""));
    assert!(json.contains("\"text\":\"ready\""));
}
