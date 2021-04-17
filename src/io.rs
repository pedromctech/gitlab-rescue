pub struct IO<A>(Box<dyn FnOnce() -> A>);

impl<A: 'static> IO<A> {
    pub fn unit(a: impl FnOnce() -> A + 'static) -> IO<A> {
        IO(Box::new(a))
    }
    pub fn map<B>(self, b: impl FnOnce(A) -> B + 'static) -> IO<B> {
        IO(Box::new(move || b(self.0())))
    }
    pub fn flat_map<B>(self, b: impl FnOnce(A) -> IO<B> + 'static) -> IO<B> {
        IO(Box::new(move || b(self.0()).0()))
    }
    pub fn apply(self) -> A {
        self.0()
    }
}
