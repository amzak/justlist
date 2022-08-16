use atty::Stream;
use serde::Deserialize;
use std::io::BufReader;

use crate::serialization::Groups;

pub struct JustListPlugin<TOptions> {
    options: TOptions,
}

pub trait JustListAction<TOptions> {
    fn execute(&self, groups: &mut Groups, options: &TOptions);
}

impl<TOptions> JustListPlugin<TOptions> {
    pub fn new(options: TOptions) -> Self {
        Self { options }
    }

    pub fn main(&self, action: &impl JustListAction<TOptions>) -> std::io::Result<()> {
        let stdin = std::io::stdin();
        let handle = stdin.lock();
        let reader = BufReader::new(handle);

        let mut groups: Groups = Groups { groups: vec![] };
        if atty::isnt(Stream::Stdin) {
            let mut de = serde_json::Deserializer::from_reader(reader);
            groups = Groups::deserialize(&mut de).unwrap();
        }

        action.execute(&mut groups, &self.options);

        let stdout = std::io::stdout();
        let stdout_handle = stdout.lock();
        let writer = std::io::BufWriter::new(stdout_handle);
        serde_json::to_writer(writer, &groups)?;

        Ok(())
    }
}
