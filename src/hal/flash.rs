pub trait Flash<W> {
    type Error;

    fn erase(&mut self, address: usize) -> Result<(), Self::Error>;
    fn program(&mut self, address: usize, words: &[W]) -> Result<(), Self::Error>;
}
