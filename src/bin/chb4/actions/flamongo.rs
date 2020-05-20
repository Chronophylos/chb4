use super::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};

lazy_static! {
    static ref FLAMONGOS: Vec<&'static str> = vec![
        "RingoDerRetardeteFlamingo",
        "RingoDerBaldImDrogenSumpfVersickendeFlamingo",
        "RingoDerFlamongo",
        "RingOhneFlamingo",
        "RingoDerDrogenAbhängigeFlamingo",
        "WehrabooFlamingo",
        "RingoDerDrogenFlamingo",
        "RingoDingoBingo",
        "RingoDrugFlamingo",
    ];
}

pub fn action() -> Arc<Action> {
    Action::with_name("flamongo")
        .regex(r"\brongo\b")
        .command(move |_context, _msg, _user| {
            let range = Uniform::new(0, FLAMONGOS.len());
            let flamongo = FLAMONGOS[range.sample(&mut thread_rng())].to_owned();

            Ok(MessageResult::MessageWithValues(
                flamongo.clone(),
                vec![flamongo],
            ))
        })
        .done()
}
