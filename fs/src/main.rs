use std::fs;
use std::io::Write;

fn main() {
    // Get the names of all the folders under ./some_dir
    let mut some_dir = fs::read_dir("./some_dir/thing").unwrap();
    let mut some_dir_names = Vec::new();
    while let Some(Ok(entry)) = some_dir.next() {
        some_dir_names.push(entry.file_name().into_string().unwrap());
    }

    // Print the names of all the folders under ./some_dir with a newline after each and a number index prefixing each
    for (i, name) in some_dir_names.iter().enumerate() {
        println!("{}: {}", i, name);
    }

    // Write the names of all the folders under ./some_dir to a file called ./pubkeys.txt, with a newline after each
    let mut file = fs::File::create("./pubkeys.txt").unwrap();
    for name in some_dir_names.iter() {
        file.write_all(name.as_bytes()).unwrap();
        file.write_all(b"\n").unwrap();
    }
}
