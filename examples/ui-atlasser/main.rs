extern crate neocogi;

use neocogi::*;
use neocogi::rs_math3d::*;

fn main() {
    let mut app = ui::App::new();
    app.run(|ctx| {
        let (_, height) = ctx.frame_size();
        ctx.window("Tools", Recti::new(0, 0, 128, height as _), ui::WidgetOption::EXPANDED | ui::WidgetOption::NO_INTERACT | ui::WidgetOption::NO_CLOSE, |ctx| {
            let r = ctx.current_rect();
            let padding = ctx.style.padding;
            //ctx.rows_with_line_config(&[1, r.width - padding * 2, 1], 0, |ctx| {
                let ctl_provider = ctx as &mut dyn ui::ControlProvider;
                for i in 0..10 {
                    //ctx.next_cell();
                    let ctl_provider = ctx as &mut dyn ui::ControlProvider;
                    if !ctl_provider.button(format!("Hello {}", i).as_str(), ui::Icon::None, ui::WidgetOption::NONE | ui::WidgetOption::ALIGN_CENTER).is_none() {
                        println!("Hello {}", i);
                    }
                    //ctx.next_cell();
                }
            //});
        });
    });
}
