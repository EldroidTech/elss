mod builder;
use std::path::Path;
use crate::builder::site_builder::SiteBuilder;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base_dir = if args.len() >= 2 { Path::new(&args[1]) } else { Path::new(".") };

    let mut site_builder = SiteBuilder::new(base_dir.to_path_buf());
    site_builder.build();
}