extern crate getopts;
extern crate image;
extern crate time;

use time::PreciseTime;
use std::default::Default;
use std::io::Read;
use std::fs::File;

pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod style;
pub mod painting;
pub mod pdf;

fn main() {
    // Parse command-line options:
    let mut opts = getopts::Options::new();
    opts.optopt("h", "html", "HTML document", "FILENAME");
    opts.optopt("c", "css", "CSS stylesheet", "FILENAME");
    opts.optopt("o", "output", "Output file", "FILENAME");
    opts.optopt("f", "format", "Output file format", "png | pdf");

    let matches = opts.parse(std::env::args().skip(1)).unwrap();
    let str_arg = |flag: &str, default: &str| -> String {
        matches.opt_str(flag).unwrap_or(default.to_string())
    };

    // Choose a format:
    let png = match &str_arg("f", "png")[..] {
        "png" => true,
        "pdf" => false,
        x => panic!("Unknown output format: {}", x),
    };

    // Read input files:
    let html = read_source(str_arg("h", "examples/test.html"));
    let css  = read_source(str_arg("c", "examples/test.css"));

    // Since we don't have an actual window, hard-code the "viewport" size.
    let mut viewport: layout::Dimensions = Default::default();
    viewport.content.width  = 800.0;
    viewport.content.height = 6000.0;

    // Parsing and rendering:

    let start = PreciseTime::now();        
    let root_node = html::parse(html);
    let parse1_time = PreciseTime::now();        
    let stylesheet = css::parse(css);
    let parse2_time = PreciseTime::now();        
    let style_root = style::style_tree(&root_node, &stylesheet);
    let style_time = PreciseTime::now();        
    let layout_root = layout::layout_tree(&style_root, viewport);
    let layout_time = PreciseTime::now();        

    // Create the output file:
    let filename = str_arg("o", if png { "output.png" } else { "output.pdf" });
    let mut file = File::create(&filename).unwrap();

    // Write to the file:
    let ok = if png {
        let canvas = painting::paint(&layout_root, viewport.content);
        let (w, h) = (canvas.width as u32, canvas.height as u32);
        let img = image::ImageBuffer::from_fn(w, h, move |x, y| {
            let color = canvas.pixels[(y * w + x) as usize];
            image::Pixel::from_channels(color.r, color.g, color.b, color.a)
        });
        image::ImageRgba8(img).save(&mut file, image::PNG).is_ok()
    } else {
        pdf::render(&layout_root, viewport.content, &mut file).is_ok()
    };
    if ok {
        println!("Saved output as {}", filename)
    } else {
        println!("Error saving output as {}", filename)
    }
    let output_time = PreciseTime::now();

    println!("parse1: {} sec", start.to(parse1_time));
    println!("parse2: {} sec", parse1_time.to(parse2_time));
    println!("style:  {} sec", parse2_time.to(style_time));
    println!("layout: {} sec", style_time.to(layout_time));
    println!("output: {} sec", layout_time.to(output_time));
}

fn read_source(filename: String) -> String {
    let mut str = String::new();
    File::open(filename).unwrap().read_to_string(&mut str).unwrap();
    str
}
