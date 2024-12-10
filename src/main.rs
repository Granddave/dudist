use std::path::Path;

use walkdir::WalkDir;

#[derive(Debug)]
struct Distribution {
    min: u64,
    max: u64,
    median: f64,
    lower_quartile: f64,
    upper_quartile: f64,
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

fn calculate_distribution(sizes: Vec<u64>) -> Distribution {
    let mut sizes = sizes;
    sizes.sort();
    let min = sizes[0];
    let max = sizes[sizes.len() - 1];
    let median = if sizes.len() % 2 == 0 {
        let mid = sizes.len() / 2;
        (sizes[mid - 1] + sizes[mid]) as f64 / 2.0
    } else {
        sizes[sizes.len() / 2] as f64
    };
    let lower_quartile = if sizes.len() % 4 == 0 {
        let mid = sizes.len() / 4;
        (sizes[mid - 1] + sizes[mid]) as f64 / 2.0
    } else {
        sizes[sizes.len() / 4] as f64
    };
    let upper_quartile = if sizes.len() % 4 == 0 {
        let mid = sizes.len() * 3 / 4;
        (sizes[mid - 1] + sizes[mid]) as f64 / 2.0
    } else {
        sizes[sizes.len() * 3 / 4] as f64
    };
    Distribution {
        min,
        max,
        median,
        lower_quartile,
        upper_quartile,
    }
}

fn print_distribution(dist: &Distribution) {
    println!("Min: {}", byte_unit::Byte::from_u64(dist.min));
    println!("Max: {}", byte_unit::Byte::from_u64(dist.max));
    println!(
        "Median: {:?}",
        byte_unit::Byte::from_f64(dist.median.round())
    );
    println!(
        "Lower Quartile: {:?}",
        byte_unit::Byte::from_f64(dist.lower_quartile.round())
    );
    println!(
        "Upper Quartile: {:?}",
        byte_unit::Byte::from_f64(dist.upper_quartile.round())
    );
}

fn plot_box_diagram(dist: &Distribution, max_value: u64) {
    let light_shade = "\u{2591}"; // use between min and lower quartile, and upper quartile and max
    let medium_shade = "\u{2592}"; // use between lower quartile and median, and median and upper quartile
    let dark_shade = "\u{2593}"; // use for median
    let cli_width = 200;

    let min = (dist.min as f64 / max_value as f64 * cli_width as f64).round() as usize;
    let lower_quartile =
        (dist.lower_quartile as f64 / max_value as f64 * cli_width as f64).round() as usize;
    let median = (dist.median as f64 / max_value as f64 * cli_width as f64).round() as usize;
    let upper_quartile =
        (dist.upper_quartile as f64 / max_value as f64 * cli_width as f64).round() as usize;
    let max = (dist.max as f64 / max_value as f64 * cli_width as f64).round() as usize;
    print!(
        "Min: {:#.2} ",
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
        " Max: {:#.2}",
        byte_unit::AdjustedByte::from(byte_unit::Byte::from_u64(dist.max))
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = Path::new(args.get(1).expect("Please provide a path"));
    let sizes = process_dir(&path);
    let dist = calculate_distribution(sizes);
    print_distribution(&dist);
    println!("{:?}", dist);
    plot_box_diagram(&dist, dist.max);
}
