#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_time::{Timer};
use esp_hal::clock::CpuClock;
use esp_hal::time::Rate;
use esp_hal::rmt::Rmt;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::rmt::{PulseCode, TxChannelCreator, TxChannelConfig, TxChannelAsync};
use esp_hal::gpio::Level;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use core::fmt::Write;

#[panic_handler]
fn panic(p: &core::panic::PanicInfo) -> ! {
    let mut usb = UsbSerialJtag::new(unsafe { esp_hal::peripherals::USB_DEVICE::steal() });

    if let Some(l) = p.location() {
        let _ = write!(usb, "{}: ", l);
    }

    let _ = write!(usb, "{}", p.message());

    loop {}
}
// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const LED_COUNT: usize = 1;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {

    let peripherals = esp_hal::init(
        esp_hal::Config::default()
            .with_cpu_clock(CpuClock::max()));

    esp_hal_embassy::init(SystemTimer::new(peripherals.SYSTIMER).alarm0);

	let _neopixel_i2c_power = Output::new(peripherals.GPIO20,
        		esp_hal::gpio::Level::High,
				OutputConfig::default());

	let mut channel = Rmt::new(peripherals.RMT, Rate::from_mhz(80))
		.unwrap()
		.into_async()
		.channel0
		.configure_tx(peripherals.GPIO9, 
			TxChannelConfig::default().with_clk_divider(1))
		.unwrap();

	// data is a sequence of 24bit GRB colors, as [u8; _]
	let mut data: [u8; 3] = [255, 0, 0];    

	spawner.spawn(task()).unwrap();

	loop {
		let mut pulses: [u32; LED_COUNT*24 + 1] = [0; _];

		data.map(byte_to_pulses).iter().enumerate().for_each(|(i, e)|
			pulses[i * 8..(i + 1) * 8].copy_from_slice(e));

		pulses[pulses.len()-1] = PulseCode::new(Level::Low, 0, Level::Low, 0);
		channel.transmit(&pulses).await.unwrap();
		// Timer::after_micros(50).await;

		// task()
		Timer::after_millis(500).await;
		// task()

		data[2] = data[2].wrapping_add(1);
	}
}

fn byte_to_pulses(b: u8) -> [u32; 8] {
    [0, 1, 2, 3, 4, 5, 6, 7].map(|i| match (b >> i) & 1 {
        0 => PulseCode::new(Level::High, 32, Level::Low, 68),
        _ => PulseCode::new(Level::High, 64, Level::Low, 36),
    })
}

#[embassy_executor::task]
async fn task() {

}
