use crate::{App0Jfif, Dqt, Dht, Frame, Dac, Scan, Rst};

pub trait Handler {
    fn handle_app(&mut self, nr: u8, data: &[u8]);

    fn handle_app0_jfif(&mut self, jfif: &App0Jfif);

    fn handle_dqt(&mut self, tables: &[Dqt]);

    fn handle_dht(&mut self, tables: &[Dht]);

    fn handle_dac(&mut self, dac: &Dac);

    fn handle_frame(&mut self, frame: &Frame);

    fn handle_scan(&mut self, scan: &Scan);

    fn handle_dri(&mut self, restart: u16);

    fn handle_rst(&mut self, restart: &Rst);

    fn handle_comment(&mut self, data: &[u8]);

    fn handle_unknown(&mut self, marker: u8, data: &[u8]);
}