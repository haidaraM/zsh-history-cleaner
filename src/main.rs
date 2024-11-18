mod errors;
mod history;

fn main() {
    let entry = history::HistoryEntry::try_from("");

    match entry {
        Ok(en) => {
            println!("{:?}", en);
        }
        Err(er) => {
            println!("{}", er);
        }
    }
}
