use std::fs::File;

mod json;

fn main() {
    let mut args = std::env::args();
    match args.next().unwrap().as_ref() {
        "layout" => {
            serde_json::from_reader::<_, json::layout::Layout>(
                File::open(args.next().unwrap()).unwrap(),
            )
            .unwrap();
        }
        "style" => {
            serde_json::from_reader::<_, json::style::Style>(
                File::open(args.next().unwrap()).unwrap(),
            )
            .unwrap();
        }
        _ => {}
    }
}
