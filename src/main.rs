mod errors;
mod history;

fn main() {
    let entry = history::HistoryEntry::try_from("fake");

    match entry {
        Ok(en) => {
            println!("{:?}", en);
        }
        Err(er) => {
            println!("Error when parsing {:?}", er);
        }
    }
}
