#![no_std]
#![no_main]
#![allow(dead_code)]
#![deny(clippy::unwrap_used)]

use crate::rom::i2c;
use core::cell::OnceCell;
use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};
use cortex_m::interrupt::CriticalSection;
use cortex_m_rt::{entry, exception};
use disp::Character;
use iic::I2C1State;
use lpc11u6x_pac::{interrupt, Interrupt, NVIC};

pub const FIRMWARE_VERSION: u8 = 1;

static mut INTERRUPT_CTX: OnceCell<InterruptContext> = OnceCell::new();

mod adc;
mod clock;
mod dbg;
mod disp;
mod iic;
mod pins;
mod rom;
mod settings;
mod setup;
mod timer;
mod uart0;
mod vrm;

#[entry]
fn main() -> ! {
    let core_periph =
        lpc11u6x_pac::CorePeripherals::take().expect("CorePeripherals have not been taken yet");
    let periph = lpc11u6x_pac::Peripherals::take().expect("Peripherals have not been taken yet");

    let iocon = periph.IOCON;
    let sys = periph.SYSCON;
    let gpio = periph.GPIO_PORT;
    let pint = periph.PINT;
    let nvic = core_periph.NVIC;
    let gint0 = periph.GINT0;

    setup::enable_clocks(&sys.sysahbclkctrl);
    setup::setup_io_pins(&iocon, &gpio);

    let debug_out = dbg::init_debug_logging(&sys, periph.USART0);
    debug_out.debug("Epower_V epsilon firmware version 1.0.0");
    debug_out.debug("Epower V project team: Illya Tsemenko, Vincent Lucido, Steven Chen, Link Kuo, Maggie Chan, Jason Lin, Jacob Freeman, Johnny Wang, Pagan Tsai, Darren Hsu, Helen Hu, Kaysta Lin, Wade Chen, Sandy Yeh, Nicole Chen, and many others :)");
    debug_out.debug("Application code running");

    debug_out.debug("Clocking up to 48MHz");
    clock::set_48_mhz(&sys);
    debug_out.reinit(&sys);

    sys.pdruncfg
        .modify(|_r, w| w.usbpad_pd().powered().usbpll_pd().powered());

    // reset i2c
    sys.presetctrl
        .modify(|_r, w| w.i2c0_rst_n().reset().i2c1_rst_n().reset());
    sys.presetctrl
        .modify(|_r, w| w.i2c0_rst_n().clear_reset().i2c1_rst_n().clear_reset());

    periph
        .I2C0
        .conclr
        .write(|w| unsafe { w.bits(1 << 2 | 1 << 3 | 1 << 5 | 1 << 6) });
    periph
        .I2C1
        .conclr
        .write(|w| unsafe { w.bits(1 << 2 | 1 << 3 | 1 << 5 | 1 << 6) });
    let i2c0 = unsafe { i2c::I2CDriver::new(crate::i2c::Bus::I2C0) };

    i2c0.set_bitrate(clock::get_sys_clk(&sys), 1_000_000);




    let pin_sel = &sys.pintsel;
    unsafe {
        pin_sel[2].write(|w| w.intpin().bits(45)); // connect int2 to PIO1_21
        pin_sel[3].write(|w| w.intpin().bits(52)); // connect int3 to PIO1_28
        pin_sel[4].write(|w| w.intpin().bits(8)); // connect int4 to PIO0_8
        gint0.port_ena[2].write(|w| w.bits(0x8000)); // connect gpint1 to PI02_15
        gint0.port_pol[2].write(|w| w.bits(0)); // low level trigger
        gint0.ctrl.write(|w| w.bits(1)); // reset detection
        pin_sel[6].write(|w| w.intpin().bits(54)); // connect int6 to PIO1_30
    }
    pint.ienf.write(|w| {
        w.enaf2()
            .set_bit()
            .enaf3()
            .set_bit()
            .enaf4()
            .set_bit()
            .enaf6()
            .set_bit()
    }); // select falling edge

    pint.fall.write(|w| {
        w.fdet2()
            .set_bit()
            .fdet3()
            .set_bit()
            .fdet4()
            .set_bit()
            .fdet6()
            .set_bit()
    }); // reset detection

    let adc = adc::Adc::new(periph.ADC, &sys);
    

    let counter = timer::enable_timer(&sys, timer::TimerSel::CT16B1, periph.CT16B1);
    let freq = clock::get_sys_clk(&sys);

    let mut display = disp::Display::new(&gpio, &sys, &i2c0, periph.CT16B0);

    let settings = settings::Settings::new(sys);

    display.set_display_orientation(settings.get_display_orientation());

    let i2c1 = periph.I2C1;
    iic::init_slave_recv(&i2c1);
    let i2c1_state = I2C1State::new(i2c1);

    vrm::enable_l2_en(&i2c0);
    if settings.is_l1_enabled() {
        vrm::enable_l1(&gpio);
    }
    vrm::enable_dvid(&i2c0);
    if settings.is_l2_enabled() {
        vrm::enable_l2(&gpio);
    }
    let l1_voltage_raw = settings.l1_boot_voltage();
    vrm::set_voltage_l1_raw(&i2c0, l1_voltage_raw);

    let l2_voltage_raw = settings.l2_boot_voltage();
    vrm::set_voltage_l2_raw(&i2c0, l2_voltage_raw);

    unsafe {
        INTERRUPT_CTX
            .set(InterruptContext {
                dbg: debug_out,
                nvic,
                pint,
                gint0,
                display,
                i2c1_state,
                i2c_driver: i2c0,
                settings,
                gpio,
                adc,
            })
            .map_err(|_| ())
            .expect("Interrupt Context was not set already");
    }

    unsafe {
        NVIC::unmask(lpc11u6x_pac::Interrupt::GINT0);
        NVIC::unmask(lpc11u6x_pac::Interrupt::PIN_INT2);
        NVIC::unmask(lpc11u6x_pac::Interrupt::PIN_INT3);
        NVIC::unmask(lpc11u6x_pac::Interrupt::PIN_INT4);
        NVIC::unmask(lpc11u6x_pac::Interrupt::PIN_INT6);
        NVIC::unmask(lpc11u6x_pac::Interrupt::CT16B0);
        NVIC::unmask(lpc11u6x_pac::Interrupt::CT16B1);
        NVIC::unmask(lpc11u6x_pac::Interrupt::I2C1);
        NVIC::unmask(interrupt::ADC_A);
    }

    counter.reset();
    counter.int_on_match(2);
    counter.set_prescale(0xFFFF);
    counter.set_match(2, (freq >> 16) as u16);
    counter.reset_on_match(2);
    counter.start();
    //unsafe { i2c_proxy::i2c_logic() };
    loop {
        cortex_m::asm::wfi();
    }
}

enum InterruptSource {
    BtnUP,
    BtnRight,
    BtnDown,
    BtnLeft,
    BtnEnter,
    BtnReturn,
    BtnG0,
    Timer16B0,
    Timer16B1,
    I2C1,
    Other(i16),
}

struct InterruptContext {
    dbg: dbg::Dbg,
    nvic: lpc11u6x_pac::NVIC,
    pint: lpc11u6x_pac::PINT,
    gint0: lpc11u6x_pac::GINT0,
    display: disp::Display,
    i2c1_state: I2C1State,
    i2c_driver: i2c::I2CDriver,
    settings: settings::Settings,
    gpio: lpc11u6x_pac::GPIO_PORT,
    adc: adc::Adc,
}

fn on_interrupt(_cs: &CriticalSection, source: InterruptSource) {
    let ctx = unsafe {
        INTERRUPT_CTX
            .get_mut()
            .expect("Interrupt Context has been set")
    };

    match source {
        InterruptSource::BtnUP => {
            ctx.pint.fall.write(|w| w.fdet2().set_bit());
        }
        InterruptSource::BtnRight => {
            ctx.pint.fall.write(|w| w.fdet0().set_bit());
        }
        InterruptSource::BtnDown => {
            ctx.pint.fall.write(|w| w.fdet4().set_bit());
        }
        InterruptSource::BtnLeft => {
            ctx.pint.fall.write(|w| w.fdet5().set_bit());
        }
        InterruptSource::BtnEnter => {
            ctx.pint.fall.write(|w| w.fdet3().set_bit());
            ctx.display.set_char(0, Character::Nine);
            ctx.display.set_char(1, Character::A);
            ctx.display.set_char(2, Character::B);
            ctx.display.set_char(3, Character::C);
            ctx.display.set_char(4, Character::D);
            ctx.display.set_char(5, Character::E);
            ctx.display.set_char(6, Character::F);
            ctx.display.set_char(7, Character::Dot);
        }
        InterruptSource::BtnReturn => {
            ctx.pint.fall.write(|w| w.fdet6().set_bit());
            /*
            ctx.display
                .set_all([1, 2, 4, 8, 16, 32, 64, 128]);*/
            ctx.gpio.dir[2].modify(|_r, w| w.dirp2().clear_bit());
        }
        InterruptSource::BtnG0 => {
            unsafe {
                ctx.gint0.ctrl.write(|w| w.bits(1)); // reset detection
            }
        }
        InterruptSource::Timer16B0 => {
            ctx.display.increment_digit(&ctx.gpio, &ctx.i2c_driver);
        }
        InterruptSource::Timer16B1 => {
            unsafe {
                lpc11u6x_pac::Peripherals::steal() // change
                    .CT16B1
                    .ir
                    .write(|w| w.bits(4));
            }

            match ctx.settings.get_disp_top_mode() {
                0 => {
                    ctx.display.set_row_top(disp::voltage_into_row(
                        vrm::read_voltage_l1(&ctx.i2c_driver).unwrap_or(0.0),
                    ));
                }
                1 => {
                    ctx.display.set_row_top(disp::current_into_row(
                        vrm::read_current_l1(&ctx.i2c_driver).unwrap_or(0),
                    ));
                }
                2 => {
                    ctx.display.set_row_top(disp::temp_into_row(
                        vrm::read_temp_l1(&ctx.i2c_driver).unwrap_or(0),
                    ));
                }
                3 => {
                    ctx.display.set_row_top(disp::voltage_into_row(
                        vrm::read_voltage_l2(&ctx.i2c_driver).unwrap_or(0.0),
                    ));
                }
                4 => {
                    ctx.display.set_row_top(disp::current_into_row(
                        vrm::read_current_l2(&ctx.i2c_driver).unwrap_or(0),
                    ));
                }
                5 => {
                    ctx.display.set_row_top(disp::temp_into_row(
                        vrm::read_temp_l2(&ctx.i2c_driver).unwrap_or(0),
                    ));
                }
                _ => {
                    ctx.display.set_row_top([
                        Character::F,
                        Character::F,
                        Character::F,
                        Character::F,
                    ]);
                }
            }
            match ctx.settings.get_disp_bottom_mode() {
                0 => {
                    ctx.display.set_row_bottom(disp::voltage_into_row(
                        vrm::read_voltage_l1(&ctx.i2c_driver).unwrap_or(0.0),
                    ));
                }
                1 => {
                    ctx.display.set_row_bottom(disp::current_into_row(
                        vrm::read_current_l1(&ctx.i2c_driver).unwrap_or(0),
                    ));
                }
                2 => {
                    ctx.display.set_row_bottom(disp::temp_into_row(
                        vrm::read_temp_l1(&ctx.i2c_driver).unwrap_or(0),
                    ));
                }
                3 => {
                    ctx.display.set_row_bottom(disp::voltage_into_row(
                        vrm::read_voltage_l2(&ctx.i2c_driver).unwrap_or(0.0),
                    ));
                }
                4 => {
                    ctx.display.set_row_bottom(disp::current_into_row(
                        vrm::read_current_l2(&ctx.i2c_driver).unwrap_or(0),
                    ));
                }
                5 => {
                    ctx.display.set_row_bottom(disp::temp_into_row(
                        vrm::read_temp_l2(&ctx.i2c_driver).unwrap_or(0),
                    ));
                }
                _ => {
                    ctx.display.set_row_bottom([
                        Character::F,
                        Character::F,
                        Character::F,
                        Character::F,
                    ]);
                }
            }
        }
        InterruptSource::I2C1 => ctx.i2c1_state.handle_state(
            &mut ctx.settings,
            &ctx.i2c_driver,
            &mut ctx.display,
            &ctx.adc,
            &ctx.gpio,
        ),
        InterruptSource::Other(irqn) => {
            ctx.dbg.debug("Unhandled Interrupt: ");
            ctx.dbg.debug_number(irqn as u32);
        }
    }
}

#[interrupt]
fn GINT0() {
    cortex_m::interrupt::free(|cs| {
        lpc11u6x_pac::NVIC::unpend(lpc11u6x_pac::Interrupt::GINT0);
        on_interrupt(cs, InterruptSource::BtnG0);
    });
}

//RIGHT
#[interrupt]
fn PIN_INT1() {
    cortex_m::interrupt::free(|cs| {
        lpc11u6x_pac::NVIC::unpend(lpc11u6x_pac::Interrupt::PIN_INT0);
        on_interrupt(cs, InterruptSource::BtnRight);
    });
}
//UP
#[interrupt]
fn PIN_INT2() {
    cortex_m::interrupt::free(|cs| {
        lpc11u6x_pac::NVIC::unpend(Interrupt::PIN_INT2);
        on_interrupt(cs, InterruptSource::BtnUP);
    });
}

//ENTER
#[interrupt]
fn PIN_INT3() {
    cortex_m::interrupt::free(|cs| {
        lpc11u6x_pac::NVIC::unpend(lpc11u6x_pac::Interrupt::PIN_INT3);
        on_interrupt(cs, InterruptSource::BtnEnter);
    });
}
//DOWN
#[interrupt]
fn PIN_INT4() {
    cortex_m::interrupt::free(|cs| {
        lpc11u6x_pac::NVIC::unpend(lpc11u6x_pac::Interrupt::PIN_INT4);
        on_interrupt(cs, InterruptSource::BtnDown);
    });
}

//LEFT
#[interrupt]
fn PIN_INT5() {
    cortex_m::interrupt::free(|cs| {
        lpc11u6x_pac::NVIC::unpend(lpc11u6x_pac::Interrupt::PIN_INT5);
        on_interrupt(cs, InterruptSource::BtnLeft);
    });
}

//RETURN
#[interrupt]
fn PIN_INT6() {
    cortex_m::interrupt::free(|cs| {
        lpc11u6x_pac::NVIC::unpend(lpc11u6x_pac::Interrupt::PIN_INT6);
        on_interrupt(cs, InterruptSource::BtnReturn);
    });
}

#[interrupt]
fn CT16B0() {
    cortex_m::interrupt::free(|cs| {
        lpc11u6x_pac::NVIC::unpend(lpc11u6x_pac::Interrupt::CT16B0);
        on_interrupt(cs, InterruptSource::Timer16B0);
    });
}

#[interrupt]
fn CT16B1() {
    cortex_m::interrupt::free(|cs| {
        lpc11u6x_pac::NVIC::unpend(lpc11u6x_pac::Interrupt::CT16B1);
        on_interrupt(cs, InterruptSource::Timer16B1);
    });
}

#[interrupt]
fn I2C1() {
    cortex_m::interrupt::free(|cs| {
        on_interrupt(cs, InterruptSource::I2C1);
    });
}

#[exception]
unsafe fn DefaultHandler(irqn: i16) {
    cortex_m::interrupt::free(|cs| {
        on_interrupt(cs, InterruptSource::Other(irqn));
    });
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
