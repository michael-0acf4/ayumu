mod basics;
mod sqlite;

fn list_string(ss: &[&str]) -> Vec<String> {
    ss.iter().map(|s| s.to_string()).collect()
}
