use ascii_osd_hud::hud::HUD;
use ascii_osd_hud::symbol::default_symbol_table;
use ascii_osd_hud::telemetry::{Telemetry, TelemetrySource};
use ascii_osd_hud::AspectRatio;
use embedded_hal::blocking::spi::{Transfer, Write};
use max7456::not_null_writer::{NotNullWriter, Screen};
use max7456::MAX7456;

// ascii-hud will generator about 120 chars, for each char
// max7456 will generate 4 byte to write, so at lease 480 bytes
// memory space is required
static mut DMA_BUFFER: [u8; 1000] = [0u8; 1000];

pub struct Max7456AsciiHud<'a, BUS, C> {
    hud: HUD<'a>,
    max7456: MAX7456<BUS>,
    dma_consumer: C,
}

pub struct StubTelemetrySource {}

impl TelemetrySource for StubTelemetrySource {
    fn get_telemetry(&self) -> Telemetry {
        Default::default()
    }
}

impl<'a, E, BUS, C> Max7456AsciiHud<'a, BUS, C>
where
    BUS: Write<u8, Error = E> + Transfer<u8, Error = E>,
    C: Fn(&[u8]),
{
    pub fn new(telemetry: &'a dyn TelemetrySource, max7456: MAX7456<BUS>, dma_consumer: C) -> Self {
        let hud = HUD::new(
            telemetry,
            &default_symbol_table(),
            150,
            AspectRatio::Standard,
        );
        Self {
            hud,
            max7456,
            dma_consumer,
        }
    }

    pub fn start_draw(&mut self) {
        let mut screen = Screen::default();
        self.hud.draw(&mut screen.0);
        let mut writer = NotNullWriter::new(&screen, Default::default());
        let operations = writer.write(unsafe { &mut DMA_BUFFER }).unwrap();
        // self.max7456.write_operations(&operations);
        let consumer = &self.dma_consumer;
        consumer(operations.0);
    }
}