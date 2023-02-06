extern crate neocogi;

use neocogi::rs_math3d::*;
use neocogi::ui::Style;
use neocogi::*;
use rectangle_pack::*;

fn main() {
    let mut app = ui::App::new("atlasser example");
    let style = Style::default();
    app.run(style, |drv, ctx, style| {
        let (_, height) = ctx.frame_size();
        ctx.window(
            &style,
            "Tools",
            Recti::new(0, 0, 200, height as _),
            ui::WidgetOption::SET_SIZE | ui::WidgetOption::NO_INTERACT | ui::WidgetOption::NO_CLOSE,
            |ctx, style| {
                let r = ctx.current_rect();
                for i in 0..10 {
                    let ctl_provider = ctx as &mut dyn ui::ControlProvider;
                    if !ctl_provider
                        .button(
                            style,
                            format!("Hello {}", i).as_str(),
                            None,
                            ui::WidgetOption::NONE | ui::WidgetOption::ALIGN_CENTER,
                        )
                        .is_none()
                    {
                        println!("Hello {}", i);
                    }
                }
                style.clone()
            },
        )
        .1
        .unwrap()
    });
}
