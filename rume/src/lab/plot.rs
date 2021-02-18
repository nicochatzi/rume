use plotters::prelude::*;

/// https://coolors.co/e87461-fff07c-77bb99-97efe9-0085cc-b497d6-e8eaed-3c4353
pub mod palette {
    use plotters::prelude::*;

    #[allow(dead_code)]
    pub const GREEN: RGBColor = RGBColor(119, 187, 153);
    #[allow(dead_code)]
    pub const RED: RGBColor = RGBColor(232, 116, 97);
    #[allow(dead_code)]
    pub const YELLOW: RGBColor = RGBColor(255, 240, 124);
    #[allow(dead_code)]
    pub const LIGHT_B: RGBColor = RGBColor(151, 239, 233);
    #[allow(dead_code)]
    pub const DEEP_B: RGBColor = RGBColor(0, 133, 204);
    #[allow(dead_code)]
    pub const PURPLE: RGBColor = RGBColor(180, 151, 214);
    #[allow(dead_code)]
    pub const GREY: RGBColor = RGBColor(60, 67, 83);
    #[allow(dead_code)]
    pub const WHITE: RGBColor = RGBColor(232, 234, 237);
}

pub fn plot(buffer: &[f32], file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let x_range = 0_f64..buffer.len() as f64;
    let y_range = -1.0..1.0;

    let root = BitMapBackend::new(file_name, (1024, 768)).into_drawing_area();
    root.fill(&palette::WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .right_y_label_area_size(40)
        .margin(5)
        .build_cartesian_2d(x_range.clone(), y_range.clone())?
        .set_secondary_coord(x_range, y_range);

    chart
        .configure_mesh()
        .y_label_formatter(&|x| format!("{:e}", x))
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            buffer
                .iter()
                .enumerate()
                .map(|(i, x)| (i as f64, *x as f64)),
            &palette::DEEP_B,
        ))?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &palette::DEEP_B));

    chart
        .configure_series_labels()
        .background_style(&RGBColor(128, 128, 128))
        .draw()?;

    Ok(())
}
