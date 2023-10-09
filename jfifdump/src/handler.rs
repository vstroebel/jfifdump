use crate::{App0Jfif, Dac, Dht, Dqt, Frame, Rst, Scan};

pub trait Handler {
    fn handle_app(&mut self, position: usize, length: usize, nr: u8, data: &[u8]);

    fn handle_app0_jfif(&mut self, position: usize, length: usize, jfif: &App0Jfif);

    fn handle_dqt(&mut self, position: usize, length: usize, tables: &[Dqt]);

    fn handle_dht(&mut self, position: usize, length: usize, tables: &[Dht]);

    fn handle_dac(&mut self, position: usize, length: usize, dac: &Dac);

    fn handle_frame(&mut self, position: usize, length: usize, frame: &Frame);

    fn handle_scan(&mut self, position: usize, length: usize, scan: &Scan);

    fn handle_dri(&mut self, position: usize, length: usize, restart: u16);

    fn handle_rst(&mut self, position: usize, length: usize, restart: &Rst);

    fn handle_comment(&mut self, position: usize, length: usize, data: &[u8]);

    fn handle_unknown(&mut self, position: usize, length: usize, marker: u8, data: &[u8]);

    fn handle_eoi(&mut self, position: usize, length: usize);

    fn handle_soi(&mut self, position: usize, length: usize);
}
