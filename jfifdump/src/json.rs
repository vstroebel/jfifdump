use crate::{App0Jfif, Dac, Dht, Dqt, Frame, Handler, Rst, Scan};

use crate::reader::get_marker_string;
use json::object::Object;
use json::{object, JsonValue};

pub struct JsonFormat {
    markers: Vec<JsonValue>,
    verbose: bool,
}

impl JsonFormat {
    pub fn new(verbose: bool) -> JsonFormat {
        JsonFormat {
            markers: vec![],
            verbose,
        }
    }

    fn add(&mut self, value: Object) {
        self.markers.push(JsonValue::Object(value));
    }

    pub fn stringify(&self) -> String {
        json::stringify_pretty(JsonValue::Array(self.markers.clone()), 4)
    }
}

impl Handler for JsonFormat {
    fn handle_app(&mut self, position: usize, nr: u8, data: &[u8]) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", format!("App(0x{:X})", nr).into());

        value.insert("start", get_marker_string(data, 20).into());

        if self.verbose {
            value.insert("data", data.into());
        }

        self.add(value);
    }

    fn handle_app0_jfif(&mut self, position: usize, jfif: &App0Jfif) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "App(0x0):JFIF".into());

        let mut density = Object::new();

        match jfif.unit {
            0 => density.insert("unit", "pixel".into()),
            1 => density.insert("unit", "dpi".into()),
            2 => density.insert("unit", "dpcm".into()),
            _ => density.insert("unit", format!("unknown {}", jfif.unit).into()),
        };

        density.insert("x", jfif.x_density.into());
        density.insert("y", jfif.y_density.into());
        value.insert("density", density.into());

        let mut thumbnail = Object::new();
        thumbnail.insert("width", jfif.x_thumbnail.into());
        thumbnail.insert("height", jfif.y_thumbnail.into());

        if self.verbose {
            if let Some(data) = &jfif.thumbnail {
                thumbnail.insert("data", data.clone().into());
            }
        }

        value.insert("thumbnail", thumbnail.into());

        self.add(value);
    }

    fn handle_dqt(&mut self, position: usize, tables: &[Dqt]) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "DQT".into());

        let tables: Vec<JsonValue> = tables
            .iter()
            .map(|table| {
                let mut t_value = Object::new();
                t_value.insert("dest", table.dest.into());
                t_value.insert("precision", table.precision.into());

                if self.verbose {
                    t_value.insert("data", table.values.to_vec().into());
                }

                JsonValue::Object(t_value)
            })
            .collect();

        value.insert("tables", tables.into());

        self.add(value);
    }

    fn handle_dht(&mut self, position: usize, tables: &[Dht]) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "DHT".into());

        let tables: Vec<JsonValue> = tables
            .iter()
            .map(|table| {
                let mut t_value = Object::new();
                t_value.insert("class", table.class.into());
                t_value.insert("dest", table.dest.into());

                if self.verbose {
                    t_value.insert("code_lengths", table.code_lengths.to_vec().into());
                    t_value.insert("values", table.values.to_vec().into());
                }

                JsonValue::Object(t_value)
            })
            .collect();

        value.insert("tables", tables.into());

        self.add(value);
    }

    fn handle_dac(&mut self, position: usize, dac: &Dac) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "DAC".into());

        let params: Vec<JsonValue> = dac
            .params
            .iter()
            .map(|param| {
                object! {
                    class: param.class,
                    dest: param.dest,
                    param: param.value,
                }
            })
            .collect();

        value.insert("params", params.into());

        self.add(value);
    }

    fn handle_frame(&mut self, position: usize, frame: &Frame) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "SOF".into());
        value.insert("type", frame.get_sof_name().into());

        value.insert("precision", frame.precision.into());
        value.insert(
            "dimension",
            object! {
               width:  frame.dimension_x,
                height: frame.dimension_y,
            },
        );

        value.insert(
            "components",
            frame
                .components
                .iter()
                .map(|component| {
                    object! {
                        id: component.id,
                        sampling_facor: object! {
                            horizontal: component.horizontal_sampling_factor,
                            vertical: component.vertical_sampling_factor,
                        },
                        quantization_table: component.quantization_table,
                    }
                })
                .collect::<Vec<_>>()
                .into(),
        );

        self.add(value);
    }

    fn handle_scan(&mut self, position: usize, scan: &Scan) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "SOS".into());

        value.insert(
            "components",
            scan.components
                .iter()
                .map(|component| {
                    object! {
                        id: component.id,
                        dc_table: component.dc_table,
                        ac_table: component.ac_table,
                    }
                })
                .collect::<Vec<_>>()
                .into(),
        );

        value.insert(
            "selection",
            object! {
                start: scan.selection_start,
                end: scan.selection_end,
            },
        );

        value.insert(
            "approximation",
            object! {
                low: scan.approximation_low,
                high: scan.approximation_high,
            },
        );

        value.insert("size", scan.data.len().into());

        if self.verbose {
            value.insert("data", scan.data.clone().into());
        }

        self.add(value);
    }

    fn handle_dri(&mut self, position: usize, restart: u16) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "DRI".into());
        value.insert("restart", restart.into());

        self.add(value);
    }

    fn handle_rst(&mut self, position: usize, restart: &Rst) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", format!("RST({})", restart.nr).into());

        value.insert("size", restart.data.len().into());

        if self.verbose {
            value.insert("data", restart.data.clone().into());
        }

        self.add(value);
    }

    fn handle_comment(&mut self, position: usize, data: &[u8]) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "COM".into());

        if let Ok(comment) = std::str::from_utf8(data) {
            value.insert("text", comment.into());
        } else {
            value.insert("raw", data.into());
        }

        self.add(value);
    }

    fn handle_unknown(&mut self, position: usize, marker: u8, data: &[u8]) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", format!("Marker(0x{:X})", marker).into());

        value.insert("size", data.len().into());

        if self.verbose {
            value.insert("data", data.into());
        }
        self.add(value);
    }

    fn handle_eoi(&mut self, position: usize) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "EOI".into());
        self.add(value);
    }

    fn handle_soi(&mut self, position: usize) {
        let mut value = Object::new();
        value.insert("position", position.into());
        value.insert("marker", "SOI".into());
        self.add(value);
    }
}
