define_spis! {
    Spi1 => (gpioa, PA5, PA6, PA7, AF5, into_alternate_af5)
    Spi2 => (gpiob, PB13, PB14, PB15, AF5, into_alternate_af5)
    Spi3 => (gpioc, PC10, PC11, PC12, AF6, into_alternate_af6)
}
