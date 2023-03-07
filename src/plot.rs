//! Create svg plots of sensor data using the poloto crate

use poloto::num::timestamp::UnixTime;
use ringbuffer::{AllocRingBuffer, RingBufferExt};

type DataPoint = (UnixTime, f64);

pub fn create_svg_plot(
    buffer_temp: &AllocRingBuffer<DataPoint>,
    buffer_humdity: &AllocRingBuffer<DataPoint>,
) -> anyhow::Result<String> {
    let svg = poloto::header()
        .with_viewbox_width(1200.0)
        .with_dim([1200.0, 800.0]);
    let opt = poloto::render::render_opt()
        .with_tick_lines([true, true])
        .with_viewbox(svg.get_viewbox())
        .move_into();
    let temperature: Vec<_> = buffer_temp.iter().collect();
    let humidity: Vec<_> = buffer_humdity.iter().collect();
    let plots = poloto::plots!(
        poloto::build::plot("temperature").line(temperature),
        poloto::build::plot("humidity").line(humidity)
    );
    let svg_plot = poloto::data(plots)
        .map_opt(|_| opt)
        .build_and_label(("Temperature and Humidity", "", ""))
        .append_to(svg.light_theme())
        .render_string()?;
    Ok(svg_plot)
}
