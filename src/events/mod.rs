pub trait EventHandler {
    fn window_open(&mut self);
    fn window_close(&mut self);
    fn window_resize(&mut self, width: u32, height: u32);
    fn window_focus(&mut self);
    fn window_unfocus(&mut self);
    fn window_frame(&mut self);
    fn window_after_frame(&mut self);
}
