use cursive::views::TextView;
use cursive::Cursive;

fn main() {
    let mut siv = cursive::default();

    siv.add_global_callback('q', Cursive::quit);

    siv.add_fullscreen_layer(TextView::new(
        "Hello World!\n\
         Press q to quit the application.",
    ));

    siv.run();
}
