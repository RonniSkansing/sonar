use super::*;
use crate::server::prometheus_normalize_name;
use grafana_dashboard::panel::{
    AliasColors, Axe, Axis, GridPos, Legend, Panel, Target as PanelTarget, ToolTip,
};
use grafana_dashboard::templating::Templating;
use grafana_dashboard::Dashboard;
use grafana_dashboard::Time;
use grafana_dashboard::TimePicker;
use grafana_dashboard::Variables;
use serde_json;

// TODO an this be a into trait impl?
pub fn to_prometheus_grafana(config: &Config) -> String {
    let mut panels: Vec<Panel> = Vec::new();
    let mut panel_id = 1;

    for target in &config.targets {
        panels.push(panel_from_target(panel_id, target));
        panel_id += 1;
    }

    let dashboard = Dashboard {
        annotations: None,
        description: None,
        editable: Some(true),
        gnet_id: None,
        graph_tooltip: Some(0),
        hide_controls: None,
        id: None,
        links: Some(Vec::new()),
        panels: Some(panels),
        refresh: Some(String::from("5s")),
        schema_version: Some(22),
        style: Some(String::from("dark")),
        tags: Some(Vec::new()),
        templating: Some(Templating {
            enable: None,
            list: Vec::new(),
        }),
        time: Some(Time {
            from: Some(String::from("now-1h")),
            to: Some(String::from("now")),
        }),
        timepicker: Some(TimePicker {
            collapse: None,
            enable: None,
            notice: None,
            now: None,
            refresh_intervals: Some(
                [
                    "5s".to_string(),
                    "10s".to_string(),
                    "30s".to_string(),
                    "1m".to_string(),
                    "5m".to_string(),
                    "15m".to_string(),
                    "30m".to_string(),
                    "1h".to_string(),
                    "2h".to_string(),
                    "1d".to_string(),
                ]
                .to_vec(),
            ),
            status: None,
            r#type: None,
        }),
        title: Some(String::from("Sonar")),
        timezone: Some(String::from("")),
        uid: Some(String::from("sonar")),
        variables: Variables { list: Vec::new() },
        version: Some(1),
    };

    serde_json::to_string(&dashboard).unwrap()
}

pub fn panel_from_target(id: u32, target: &Target) -> Panel {
    let mut panel_targets: Vec<PanelTarget> = Vec::new();
    for n in [95, 99].iter() {
        let n_string = n.to_string();
        let legend_format = format!("{}{}", "p", n_string);
        let prometheus_metric_name = format!(
            "histogram_quantile(0.{}, sum(rate({}_time_ms_bucket[5m])) by (le))",
            n_string,
            prometheus_normalize_name(target.name.to_string())
        );

        panel_targets.push(PanelTarget {
            expr: Some(String::from(prometheus_metric_name)),
            interval: Some(String::from("")),
            legend_format: Some(legend_format),
            ref_id: Some(n_string),
        })
    }
    // build graph for target
    let panel = Panel {
        alert: None,
        alias_colors: Some(AliasColors {}),
        bars: Some(false),
        content: None,
        dash_length: Some(10),
        dashes: Some(false),
        datasource: Some(String::from("sonar")),
        fill: Some(1),
        fill_gradient: Some(0),
        grid_pos: Some(GridPos {
            h: Some(6),
            w: Some(8),
            x: Some(0),
            y: Some(0),
        }),
        hidden_series: Some(false),
        id: Some(id),
        legend: Some(Legend {
            avg: Some(false),
            current: Some(true),
            max: Some(false),
            min: Some(false),
            show: Some(true),
            total: Some(false),
            values: Some(true),
        }),
        lines: Some(true),
        line_width: Some(1),
        null_point_mode: Some(String::from("null")),
        mode: None,
        options: Some(Vec::new()),
        percentage: Some(false),
        pointradius: Some(2),
        points: Some(false),
        renderer: Some(String::from("flot")),
        series_overrides: Some(Vec::new()),
        space_length: Some(10),
        stack: Some(false),
        stepped_line: Some(false),
        targets: Some(panel_targets),
        thresholds: Some(Vec::new()),
        time_from: None,
        time_regions: Some(Vec::new()),
        time_shift: None,
        title: Some(target.name.clone()),
        tooltip: Some(ToolTip {
            shared: Some(true),
            sort: Some(0),
            value_type: Some(String::from("individual")),
        }),
        r#type: Some(String::from("graph")),
        xaxis: Some(Axis {
            align: None,
            align_level: None,
            buckets: None,
            mode: Some(String::from("time")),
            name: None,
            show: Some(true),
            values: Some(Vec::new()),
        }),
        yaxis: Some(Axis {
            align: Some(false),
            align_level: None,
            buckets: None,
            mode: None,
            name: None,
            show: None,
            values: None,
        }),
        yaxes: Some(
            [
                Axe {
                    hash_key: Some(String::from("object:231")),
                    format: Some(String::from("none")),
                    label: None,
                    log_base: Some(1),
                    max: None,
                    min: None,
                    show: Some(true),
                },
                Axe {
                    hash_key: Some(String::from("object:232")),
                    format: Some(String::from("short")),
                    label: None,
                    log_base: Some(1),
                    max: None,
                    min: None,
                    show: Some(true),
                },
            ]
            .to_vec(),
        ),
    };

    panel
}
