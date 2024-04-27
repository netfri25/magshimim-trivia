pub struct Defer<F: FnMut()>(pub F);

impl<F> Drop for Defer<F>
where
    F: FnMut(),
{
    fn drop(&mut self) {
        (self.0)()
    }
}
