use std::thread;
use std::{
    env,
    fs::{self, read_dir, DirEntry},
    process::exit,
    sync::mpsc,
};

fn does_file_contain_nocheckin(file: &str, nocheckin_str: &str) -> bool {
    let file_contents = fs::read_to_string(file);
    if let Ok(x) = file_contents {
        return x.contains(&nocheckin_str);
    }

    false
}

fn apply_filter(f: &DirEntry) -> bool {
    f.file_name()
        .to_str()
        .map(|x| x.starts_with("."))
        .unwrap_or(false)
}

fn walk_dir(path: &str) -> (Vec<String>, bool) {
    let dir = read_dir(path);

    let (tx, rx) = mpsc::channel();

    let mut contains_nocheckin: Vec<String> = Vec::new();

    if let Ok(dir) = dir {
        thread::scope(|s| {
            for x in dir {
                let ctx = tx.clone();
                if let Ok(x) = x {
                    if apply_filter(&x) {
                        continue;
                    }

                    let fp = x.path();

                    if x.path().is_dir() {
                        s.spawn(move || {
                            let (_, ncf) = walk_dir(&fp.to_str().unwrap());
                            ctx.send(ncf).unwrap();
                        });
                    } else if does_file_contain_nocheckin(&fp.to_str().unwrap(), "NOCHECKIN") {
                        contains_nocheckin.push(String::from(fp.to_str().unwrap()));
                        ctx.send(true).unwrap();
                        println!("{} contains no checkin", fp.to_str().unwrap());
                    }
                }
            }
        });
    }

    drop(tx);

    for r in rx {
        if r {
            return (contains_nocheckin, true);
        }
    }

    (contains_nocheckin, false)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let arg = args.get(1);

    if let Some(path) = arg {
        let (_, ncf) = walk_dir(path);
        if ncf {
            println!("No checkin found.");
            exit(1)
        }
    } else {
        println!("Usage: ./nocheckin <root>");
        exit(1)
    }
}
