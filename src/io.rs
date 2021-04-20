pub struct IO<A>(Box<dyn FnOnce() -> A>);

impl<A: 'static> IO<A> {
    /// Returns a IO object with a function inside
    ///
    /// # Example
    ///
    /// ```rust
    /// let my_io_object = IO::unit(|| println("This function is a side effect!"));
    /// ```
    ///
    pub fn unit(a: impl FnOnce() -> A + 'static) -> IO<A> {
        IO(Box::new(a))
    }
    /// Maps a `IO<A>` to `IO<B>` by applying a function to a contained function.
    /// This function can be used to compose the results of two functions.
    pub fn map<B>(self, b: impl FnOnce(A) -> B + 'static) -> IO<B> {
        IO(Box::new(move || b(self.0())))
    }
    /// Calls `b` with the wrapped value and returns the result
    pub fn flat_map<B>(self, b: impl FnOnce(A) -> IO<B> + 'static) -> IO<B> {
        IO(Box::new(move || b(self.0()).0()))
    }
    /// Apply wrapped effects
    ///
    /// # Example
    ///
    /// ```rust
    /// let my_io_object = IO::unit(|| println("This function is a side effect!"));
    /// my_io_object.map(|| println("This is another effect"));
    /// my_io_object.apply(); // This print both messages
    /// ```
    ///
    pub fn apply(self) -> A {
        self.0()
    }
}
