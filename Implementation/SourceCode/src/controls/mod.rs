pub mod orbital;

trait Controls {
    fn save(&mut self);
    fn reset(&mut self);
    fn update(&mut self);
}
