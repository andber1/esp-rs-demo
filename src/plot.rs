//! Create svg plots of sensor data using the poloto crate

use poloto_chrono::UnixTime;
use ringbuffer::{AllocRingBuffer, RingBuffer};

type DataPoint = (UnixTime, [f32; 2]);

pub fn create_svg_plot(
    buffer: &AllocRingBuffer<DataPoint>,
    index: usize,
    legend: &str,
) -> anyhow::Result<String> {
    let svg = poloto::header()
        .with_viewbox_width(1200.0)
        .with_dim([1200.0, 800.0]);
    let render_frame = poloto::render::RenderFrameBuilder::new()
        .with_tick_lines([true, true])
        .with_viewbox(svg.get_viewbox())
        .build();
    let data: Vec<_> = buffer.iter().map(|x| (x.0, x.1[index] as f64)).collect();
    let plot = poloto::build::plot(legend).line(data);
    let svg_plot = render_frame
        .data(plot)
        .build_and_label((legend, "", ""))
        .append_to(svg.light_theme())
        .render_string()?;
    Ok(svg_plot)
}
