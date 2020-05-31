use super::prelude::*;
use procinfo::pid::statm_self;
use std::error::Error;
use systemstat::{ByteSize, Platform, System};

pub fn command() -> Arc<Command> {
    Command::with_name("system")
        .alias("sysstat")
        .command(|context, _args, _msg, _user| {
            let sys = System::new();

            let (mem_proc, mem_used, mem_total) = match mem(&sys) {
                Ok(t) => t,
                Err(err) => return Err(MessageError::from(err.to_string())),
            };

            let load_avg = match sys.load_average() {
                Ok(l) => l,
                Err(err) => return Err(MessageError::from(err.to_string())),
            };

            let cpu_temp = match sys.cpu_temp() {
                Ok(l) => l,
                Err(err) => return Err(MessageError::from(err.to_string())),
            };

            let uptime = match sys.uptime() {
                Ok(u) => u,
                Err(err) => return Err(MessageError::from(err.to_string())),
            };

            Ok(MessageResult::Message(format!(
                "Memory usage: {}/{}/{} Load: {} CPU Temp: {}°C Uptime: {} System Uptime: {}",
                mem_proc.to_string_as(true),
                mem_used.to_string_as(true),
                mem_total.to_string_as(true),
                load_avg.five,
                cpu_temp,
                humantime::format_duration(truncate_duration(context.elapsed())),
                humantime::format_duration(truncate_duration(uptime)),
            )))
        })
        .about("Get information about the Bot and the Server")
        .done()
}

fn mem(sys: &System) -> std::result::Result<(ByteSize, ByteSize, ByteSize), Box<dyn Error>> {
    let mem_proc = statm_self()?.size;
    let mem_proc = ByteSize::b(mem_proc as u64);

    let memory = sys.memory()?;
    let mem_used = ByteSize::b(memory.total.as_u64() - memory.free.as_u64());

    Ok((mem_proc, mem_used, memory.total))
}
