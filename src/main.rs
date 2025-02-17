use std::path::Path;

use walkdir::WalkDir;

use libc::{ioctl, isatty, STDOUT_FILENO, TIOCGWINSZ};

pub fn get_terminal_width() -> Option<u16> {
    // Check if stdout is a terminal first to avoid unnecessary unsafe operations
    if unsafe { isatty(STDOUT_FILENO) } != 1 {
        return None;
    }

    // Isolate the unsafe code in a small, well-defined block
    let ws = unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) != 0 {
            return None;
        }
        ws
    };

    Some(ws.ws_col)
}

#[derive(Debug)]
struct Distribution {
    min: u64,
    max: u64,
    median: f64,
    lower_quartile: f64,
    upper_quartile: f64,
}

impl Distribution {
    fn from_vec(sizes: Vec<u64>) -> Distribution {
        let mut sizes = sizes;
        sizes.sort();
        Distribution {
            min: sizes[0],
            max: sizes[sizes.len() - 1],
            median: if sizes.len() % 2 == 0 {
                let mid = sizes.len() / 2;
                (sizes[mid - 1] + sizes[mid]) as f64 / 2.0
            } else {
                sizes[sizes.len() / 2] as f64
            },
            lower_quartile: if sizes.len() % 4 == 0 {
                let mid = sizes.len() / 4;
                (sizes[mid - 1] + sizes[mid]) as f64 / 2.0
            } else {
                sizes[sizes.len() / 4] as f64
            },
            upper_quartile: if sizes.len() % 4 == 0 {
                let mid = sizes.len() * 3 / 4;
                (sizes[mid - 1] + sizes[mid]) as f64 / 2.0
            } else {
                sizes[sizes.len() * 3 / 4] as f64
            },
        }
    }
}

fn process_dir(path: &Path) -> Vec<u64> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            let metadata = entry.metadata().unwrap();
            metadata.len()
        })
        .filter(|size| *size > 4096)
        .collect::<Vec<_>>()
}

fn print_distribution(dist: &Distribution) {
    println!(
        "Smallest:       {:#.2}",
        byte_unit::AdjustedByte::from(byte_unit::Byte::from_u64(dist.min))
    );
    println!(
        "Lower Quartile: {:#.2}",
        byte_unit::AdjustedByte::from(
            byte_unit::Byte::from_f64(dist.lower_quartile.round()).expect("Invalid lower quartile")
        )
    );
    println!(
        "Median:         {:#.2}",
        byte_unit::AdjustedByte::from(
            byte_unit::Byte::from_f64(dist.median.round()).expect("Invalid median")
        )
    );
    println!(
        "Upper Quartile: {:#.2}",
        byte_unit::AdjustedByte::from(
            byte_unit::Byte::from_f64(dist.upper_quartile.round()).expect("Invalid upper quartile")
        )
    );
    println!(
        "Largest:        {:#.2}",
        byte_unit::AdjustedByte::from(byte_unit::Byte::from_u64(dist.max))
    );
}

fn plot_box_diagram(dist: &Distribution, max_value: u64, width: u16) {
    let light_shade = "\u{2591}"; // use between min and lower quartile, and upper quartile and max
    let medium_shade = "\u{2592}"; // use between lower quartile and median, and median and upper quartile
    let dark_shade = "\u{2593}"; // use for median
    let cli_width = width as usize - 40;

    let min = (dist.min as f64 / max_value as f64 * cli_width as f64).round() as usize;
    let lower_quartile =
        (dist.lower_quartile as f64 / max_value as f64 * cli_width as f64).round() as usize;
    let median = (dist.median as f64 / max_value as f64 * cli_width as f64).round() as usize;
    let upper_quartile =
        (dist.upper_quartile as f64 / max_value as f64 * cli_width as f64).round() as usize;
    let max = (dist.max as f64 / max_value as f64 * cli_width as f64).round() as usize;
    print!(
        "Smallest: {:#.2} ",
        byte_unit::AdjustedByte::from(byte_unit::Byte::from_u64(dist.min))
    );
    for _ in 0..min {
        print!(" ");
    }
    for _ in min..lower_quartile {
        print!("{}", light_shade);
    }
    for _ in lower_quartile..median {
        print!("{}", medium_shade);
    }
    print!("{}", dark_shade);
    for _ in median..upper_quartile {
        print!("{}", medium_shade);
    }
    for _ in upper_quartile..max {
        print!("{}", light_shade);
    }
    for _ in max..cli_width {
        print!(" ");
    }
    println!(
        " Largest: {:#.2}",
        byte_unit::AdjustedByte::from(byte_unit::Byte::from_u64(dist.max))
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = Path::new(args.get(1).expect("Please provide a path"));
    let sizes = process_dir(&path);
    if sizes.is_empty() {
        println!("No files found in the directory");
        return;
    }
    println!("Number of files: {}", sizes.len());
    let dist = Distribution::from_vec(sizes);
    print_distribution(&dist);
    plot_box_diagram(&dist, dist.max, get_terminal_width().unwrap_or(80));
}
