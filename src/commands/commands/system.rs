use super::prelude::*;
use procinfo::pid::statm_self;
use systemstat::{ByteSize, Platform, System};

pub fn command() -> Arc<Command> {
    Command::with_name("system")
        .alias("sysstat")
        .command(|context, _args, _msg, _user| {
            let sys = System::new();

            let (mem_proc, mem_used, mem_total) =
                mem(&sys).context("Could not get memory information")?;
            let load_avg = sys.load_average().context("Could not get load average")?;
            let uptime = sys.uptime().context("Could not get system uptime")?;

            Ok(MessageResult::Message(format!(
                "Memory usage: {}/{}/{} Load: {} Uptime: {} System Uptime: {}",
                mem_proc.to_string_as(true),
                mem_used.to_string_as(true),
                mem_total.to_string_as(true),
                load_avg.five,
                humantime::format_duration(truncate_duration(context.elapsed())),
                humantime::format_duration(truncate_duration(uptime)),
            )))
        })
        .about("Get information about the Bot and the Server")
        .done()
}

fn mem(sys: &System) -> Result<(ByteSize, ByteSize, ByteSize)> {
    let mem_proc = statm_self()
        .context("Could not get process memory stats")?
        .size;
    let mem_proc = ByteSize::b(mem_proc as u64);

    let memory = sys
        .memory()
        .context("Could not get sysmtem memory information")?;
    let mem_used = ByteSize::b(memory.total.as_u64() - memory.free.as_u64());

    Ok((mem_proc, mem_used, memory.total))
}
