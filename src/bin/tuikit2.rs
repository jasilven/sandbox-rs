use tuikit::prelude::*;

struct Model(String);

impl Draw for Model {
    fn draw(&self, canvas: &mut dyn Canvas) -> Result<()> {
        let (width, height) = canvas.size()?;
        let message_width = self.0.len();
        let left = (width - message_width) / 2;
        let top = height / 2;
        let _ = canvas.print(top, left, &self.0);
        Ok(())
    }
}

impl Widget for Model {}

fn main() {
    let term = Term::with_height(TermHeight::Percent(50)).unwrap();
    let model = Model("middle!".to_string());

    while let Ok(ev) = term.poll_event() {
        if let Event::Key(Key::Char('q')) = ev {
            break;
        }
        let _ = term.print(0, 0, "press 'q' to exit");

        let hsplit = HSplit::default()
            .split(
                VSplit::default()
                    .basis(Size::Percent(30))
                    .split(Win::new(&model).border(true).basis(Size::Percent(30)))
                    .split(Win::new(&model).border(true).basis(Size::Percent(30))),
            )
            .split(Win::new(&model).border(true));

        let _ = term.draw(&hsplit);
        let _ = term.present();
    }
}
