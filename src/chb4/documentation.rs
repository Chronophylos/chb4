pub trait Documentation {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn aliases(&self) -> Option<String>;
    fn regex(&self) -> Option<String>;
    fn chainable(&self) -> String;
    fn whitelisted(&self) -> String;

    fn documentation(&self) -> String {
        let aliases = match self.aliases() {
            Some(s) => format!("Aliases: {}\n", s),
            None => String::new(),
        };

        let regex = match self.regex() {
            Some(s) => format!("Regex: `{}`\n", s),
            None => String::new(),
        };

        format!(
            "\
= {}

{}{}

{}

|===
| chainable
| {}

| whitelisted
| {}
|===
            ",
            self.name(),
            aliases,
            regex,
            self.description(),
            self.chainable(),
            self.whitelisted()
        )
    }

    fn about(&self) -> String {
        let alternate = String::new(); // todo
        format!("{}{}: {}", self.name(), alternate, self.description())
    }
}
