extern crate flexi_logger;
extern crate yansi;

pub fn format(
    w: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &flexi_logger::Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        " {} {:<5} {} > {}",
        now.now().format("%Y-%m-%d %H:%M:%S"),
        style_level(level, record.level()),
        yansi::Paint::new(record.module_path().unwrap_or("<unnamed>")).bold(),
        style_message(level, &record.args())
    )
}

fn style_level<T>(level: log::Level, item: T) -> yansi::Paint<T> {
    match level {
        log::Level::Error => yansi::Paint::red(item).bold(),
        log::Level::Warn => yansi::Paint::yellow(item).bold(),
        log::Level::Info => yansi::Paint::green(item),
        log::Level::Debug => yansi::Paint::blue(item),
        log::Level::Trace => yansi::Paint::fixed(5, item),
    }
}

fn style_message<T>(level: log::Level, item: T) -> yansi::Paint<T> {
    match level {
        log::Level::Error => yansi::Paint::red(item),
        log::Level::Warn => yansi::Paint::yellow(item),
        log::Level::Info => yansi::Paint::white(item),
        log::Level::Debug => yansi::Paint::fixed(8, item),
        log::Level::Trace => yansi::Paint::fixed(8, item).italic(),
    }
}
