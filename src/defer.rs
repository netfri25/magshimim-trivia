pub struct Defer<F: FnOnce()>(pub F);

impl<F> Drop for Defer<F>
where
    F: FnOnce()
{
    fn drop(&mut self) {
        (self.0)()
    }
}
