use std::{path::{PathBuf}, fs, str, process::exit};
use clap::{Parser, arg};

use id3::{Tag, TagLike, Error, ErrorKind};

use colored::Colorize;

const FORBIDDEN_SYMBOLS: [char; 9] = [ '<', '>', ':', '\"', '/', '\\', '|', '?', '*' ];
const RESERVED_WINDOWS_NAMES: [&str; 22] = [ "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9" ];

#[derive(Parser, Debug)]
struct CliArgs {
    /// Tells application to delete original file after successful copying
    #[arg(short, long)]
    remove_source_file: bool,

    /// Directory where to put audio files
    #[arg(short, long, value_name = "DIRECTORY")]
    target_directory: PathBuf,

    /// List of original files, that needs to be managed
    files: Vec<PathBuf>,
}

struct RequiredTags {
    pub artist: String,
    pub album: String,
    pub title: String,
}

// TODO: clap - set `files` to be non-empty?
fn main() {
    let args = CliArgs::parse();
    if args.files.len() == 0 {
        println!("{} You must specify at least one file!", "ERROR!".red().bold());
        exit(2);
    }

    let file_path_pretty_print: fn(&PathBuf) -> &str = |path| {
        path.file_name().and_then(|fname| { fname.to_str() }).unwrap_or("<N/A>")
    };

    let longest_filename_len = args.files.iter()
        .map(|fname| { file_path_pretty_print(fname) })
        .map(|fname| { fname.chars().count() })
        .max()
        .unwrap_or(1);


    for file in args.files {
        let file_result = handle_file(&file, &args.target_directory);

        let filename = file_path_pretty_print(&file);
        print_handling_status(filename, longest_filename_len, &file_result);

        if args.remove_source_file && file_result.is_ok() {
            let removal_result = fs::remove_file(file);
            if removal_result.is_err() {
                let err = removal_result.unwrap_err();
                println!(
                    "{} Original file wan't removed due to error: {}",
                    "Warning!".yellow().bold(),
                    err
                );
            }
        }
    }
}


fn handle_file(filepath: &PathBuf, root_folder: &PathBuf) -> Result<(), Vec<id3::Error>> {
    let tag_result = Tag::read_from_path(filepath);
    if tag_result.is_err() {
        return Result::Err(vec![tag_result.err().unwrap()]);
    }

    let tag = tag_result.unwrap();
    let required_tags = vec![
        tag.album_artist().ok_or(Error::new(ErrorKind::NoTag, "No album artist found")),
        tag.album().ok_or(Error::new(ErrorKind::NoTag, "No album found")),
        tag.title().ok_or(Error::new(ErrorKind::NoTag, "No title found")),
    ];

    handle_tags(required_tags)
        .map(|tags| {
            if let [artist, album, title] = &tags[..] {
                RequiredTags {
                    album: (*album).clone(),
                    artist: (*artist).clone(),
                    title: (*title).clone(),
                }
            } else {
                panic!("Tags amount does not match expected (3).")
            }
        })
        .and_then(|tags| {
            let target_path = generate_target_path(filepath, root_folder, tags);
            copy_file(filepath, &target_path).map_err(|e| { vec![e] })
        })
}


fn handle_tags(tags: Vec<Result<&str, Error>>) -> Result<Vec<String>, Vec<Error>> {
    let mut result = Result::Ok(Vec::new());

    for current_tag in tags {
        match result {
            Ok(mut tags) => {
                match current_tag {
                    Ok(tag_value) => {
                        tags.push(String::from(tag_value));
                        result = Ok(tags);
                    },
                    Err(tag_error) => {
                        result = Err(vec![tag_error]);
                    }
                }
            },
            Err(mut errors) => {
                match current_tag {
                    Ok(_) => {
                        result = Err(errors);
                    },
                    Err(tag_error) => {
                        errors.push(tag_error);
                        result = Err(errors);
                    }
                }
            }
        }
    }

    result
}

fn generate_target_path(source: &PathBuf, root_folder: &PathBuf, tags: RequiredTags) -> PathBuf {
    let mut result_path = PathBuf::new();
    result_path.push(root_folder);
    result_path.push(normalize_path_entry(tags.artist.as_str()));
    result_path.push(normalize_path_entry(tags.album.as_str()));

    let mut full_target_filename = normalize_path_entry(tags.title.as_str());
    let ext = source.extension().map(|ext| { ext.to_str().unwrap() });
    if ext.is_some() {
        full_target_filename.push_str(".");
        full_target_filename.push_str(ext.unwrap());
    }

    result_path.push(full_target_filename);
    result_path
}

fn normalize_path_entry(path_entry: &str) -> String {
    // Based on: https://stackoverflow.com/a/31976060
    let mut result = path_entry.to_string()
        .replace(FORBIDDEN_SYMBOLS, "_")
        .replace(Vec::from_iter((0..=31).map(|b| { char::from_u32(b).unwrap()})).as_slice(), "");

    let extension_separator = ".";
    let split_by_separator = result.splitn(2, extension_separator).collect::<Vec<&str>>();
    let filename = split_by_separator.first().unwrap();

    if RESERVED_WINDOWS_NAMES.contains(filename) {
        result = result.replacen(filename, "_", 1);
    }

    result
}

fn copy_file(source: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    target.parent()
        .ok_or(Error::new(ErrorKind::InvalidInput, format!("Unexpected error while copying file to target '{}'", target.to_str().unwrap())))
        .and_then(|parent_dir| {
            fs::create_dir_all(parent_dir).map_err(|io_err| { Error::new(ErrorKind::Io(io_err), format!("Cannot create directory '{}'", parent_dir.to_str().unwrap())) })
        })
        .and_then(|()| {
            fs::copy(source, target)
                .map(|_| { () })
                .map_err(|io_err| { Error::new(ErrorKind::Io(io_err), "Failed to copy file") })
        })
}

fn print_handling_status(filename: &str, longest_filename_len: usize, result: &Result<(), Vec<id3::Error>>) {
    // Even for longest filename need to add '...'
    let dots_amount = longest_filename_len - filename.chars().count() + 10;
    let dots: String = std::iter::repeat(".").take(dots_amount).collect();

    match result {
        Ok(()) => {
            println!(
                "{}{}{}",
                filename,
                dots,
                "Ok".green().bold()
            );

        },
        Err(errors) => {
            let pretty_error_print: fn(&Error) -> String = |err| {
                format!("{}: {}", err.kind.to_string(), err.description)
            };

            let (fst, other) = errors.split_first().unwrap();
            println!(
                "{}{}{}",
                filename,
                dots,
                pretty_error_print(fst).red().bold()
            );
            
            let indent: String = std::iter::repeat(" ").take(filename.chars().count() + dots_amount).collect();
            for err in other {
            println!(
                "{}{}",
                indent,
                pretty_error_print(err).red().bold()
            );

            }
        }
    }
}