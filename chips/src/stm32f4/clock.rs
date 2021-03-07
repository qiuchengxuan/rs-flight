use drone_core::fib::ThrFiberFuture;
use drone_core::reg::prelude::*;
use drone_cortexm::{fib, reg::prelude::*};
use drone_stm32_map::reg;

pub const SYSCLK: u32 = 168_000_000;
pub const HCLK: u32 = SYSCLK;
pub const PCLK1: u32 = 42_000_000;
pub const PCLK2: u32 = 84_000_000;
const HPRE: u32 = 1; // = SYSCLK
const PPRE1: u32 = 0b101; // SYSCLK / 4 = 42MHz
const PPRE2: u32 = 0b100; // SYSCLK / 2 = 84MHz
const RTCPRE: u32 = 8; // HSE / 8 = 1MHz
const PLL_SELECTED: u32 = 0b10;
const FLASH_LATENCY: u32 = (SYSCLK - 1) / 30_000_000;

type RccRegs = (reg::rcc::Cfgr<Srt>, reg::rcc::Cr<Srt>, reg::rcc::Pllcfgr<Srt>);

pub async fn setup_pll(
    thread: &mut impl ThrFiberFuture,
    cir: reg::rcc::Cir<Crt>,
    regs: RccRegs,
    flash_acr: &reg::flash::Acr<Srt>,
) {
    let (cfgr, cr, pllcfgr) = regs;

    cir.modify(|r| r.set_hserdyie().set_pllrdyie());

    let reg::rcc::Cir { hserdyc, hserdyf, .. } = cir;

    let hse_ready = thread.add_future(fib::new_fn(move || {
        if !hserdyf.read_bit() {
            return fib::Yielded(());
        }
        hserdyc.set_bit();
        fib::Complete(())
    }));
    cr.modify(|r| r.set_hseon());
    hse_ready.await;

    flash_acr.modify(|r| r.write_latency(FLASH_LATENCY));

    // PLL = (8MHz / M) * N / P = (8MHz / 8) * 336 / 2 = 168MHz
    pllcfgr.modify(|r| r.write_pllm(8).write_plln(336).write_pllp(0).write_pllq(7).set_pllsrc());
    cr.modify(|r| r.set_pllon());
    let reg::rcc::Cir { pllrdyc, pllrdyf, .. } = cir;
    let pll_ready = thread.add_future(fib::new_fn(move || {
        if !pllrdyf.read_bit() {
            return fib::Yielded(());
        }
        pllrdyc.set_bit();
        fib::Complete(())
    }));
    pll_ready.await;

    cfgr.modify(|r| r.write_hpre(HPRE).write_ppre1(PPRE1).write_ppre2(PPRE2).write_rtcpre(RTCPRE));
    cfgr.modify(|r| r.write_sw(PLL_SELECTED));
}
