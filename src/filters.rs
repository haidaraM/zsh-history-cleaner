pub struct Filter {
    words: Vec<String>,
    ignore_case: bool,
}

impl Filter {
    pub fn new(words: &[String], ignore_case: bool) -> Self {
        Self {
            words: words.to_vec(),
            ignore_case,
        }
    }

    pub fn matches(&self, command: &str) -> bool {
        if self.ignore_case {
            let command = command.to_lowercase();
            self.words.iter().any(|word| command.contains(&word.to_lowercase()))
        } else {
            self.words.iter().any(|word| command.contains(word))
        }
    }
}
