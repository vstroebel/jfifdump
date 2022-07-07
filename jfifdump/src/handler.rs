use crate::{App0Jfif, Dac, Dht, Dqt, Frame, Rst, Scan};

pub trait Handler {
    fn handle_app(&mut self, position: usize, nr: u8, data: &[u8]);

    fn handle_app0_jfif(&mut self, position: usize, jfif: &App0Jfif);

    fn handle_dqt(&mut self, position: usize, tables: &[Dqt]);

    fn handle_dht(&mut self, position: usize, tables: &[Dht]);

    fn handle_dac(&mut self, position: usize, dac: &Dac);

    fn handle_frame(&mut self, position: usize, frame: &Frame);

    fn handle_scan(&mut self, position: usize, scan: &Scan);

    fn handle_dri(&mut self, position: usize, restart: u16);

    fn handle_rst(&mut self, position: usize, restart: &Rst);

    fn handle_comment(&mut self, position: usize, data: &[u8]);

    fn handle_unknown(&mut self, position: usize, marker: u8, data: &[u8]);
}
