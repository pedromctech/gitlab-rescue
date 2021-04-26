pub struct IO<A>(Box<dyn FnOnce() -> A>);

impl<A: 'static> IO<A> {
    /// Returns a IO object with a function inside
    ///
    /// # Example
    ///
    /// ```rust
    /// use gitlab_rescue::io::IO;
    ///
    /// let my_io_object = IO::unit(|| println!("This function is a side effect!"));
    /// ```
    ///
    pub fn unit(a: impl FnOnce() -> A + 'static) -> IO<A> {
        IO(Box::new(a))
    }

    /// Maps a `IO<A>` to `IO<B>` by applying a function to a contained function.
    /// This function can be used to compose the results of two functions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gitlab_rescue::io::IO;
    ///
    /// let io_unit = IO::unit(|| println!("This function is a side effect!"))
    ///     .map(|_| println!("This is another effect")); // Nothing happens here
    /// ```
    ///
    pub fn map<B>(self, b: impl FnOnce(A) -> B + 'static) -> IO<B> {
        IO(Box::new(move || b(self.0())))
    }

    /// Maps a `IO<A>` to `IO<B>` by applying a function to a contained function.
    /// This function can be used to compose the results of two functions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gitlab_rescue::io::IO;
    ///
    /// let io_unit = IO::unit(|| println!("This function is a side effect!"))
    ///     .flat_map(|_| IO::unit(|| println!("This is another effect in another IO unit"))); // Nothing happens here
    /// ```
    ///
    pub fn flat_map<B>(self, b: impl FnOnce(A) -> IO<B> + 'static) -> IO<B> {
        IO(Box::new(move || b(self.0()).0()))
    }

    /// Apply wrapped effects
    ///
    /// # Example
    ///
    /// ```rust
    /// use gitlab_rescue::io::IO;
    ///
    /// let my_io_object = IO::unit(|| println!("This function is a side effect!")).map(|_| println!("This is another effect"));
    /// my_io_object.apply(); // This print both messages
    /// ```
    ///
    pub fn apply(self) -> A {
        self.0()
    }
}
