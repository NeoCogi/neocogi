extern crate neocogi;

use neocogi::rs_math3d::*;
use neocogi::*;
use rectangle_pack::*;

fn main() {
    let mut app = ui::App::new();
    app.run(|ctx| {
        let (_, height) = ctx.frame_size();
        ctx.window(
            "Tools",
            Recti::new(0, 0, 200, height as _),
            ui::WidgetOption::SET_SIZE | ui::WidgetOption::NO_INTERACT | ui::WidgetOption::NO_CLOSE,
            |ctx| {
                let r = ctx.current_rect();
                for i in 0..10 {
                    let ctl_provider = ctx as &mut dyn ui::ControlProvider;
                    if !ctl_provider
                        .button(
                            format!("Hello {}", i).as_str(),
                            None,
                            ui::WidgetOption::NONE | ui::WidgetOption::ALIGN_CENTER,
                        )
                        .is_none()
                    {
                        println!("Hello {}", i);
                    }
                }
            },
        );
    });
}
