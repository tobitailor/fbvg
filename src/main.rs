use std::collections::HashMap;
use std::env;
use std::fs;

use framebuffer::Framebuffer;
use resvg::tiny_skia::{ Pixmap, Transform };
use resvg::usvg::{ fontdb, Options, Tree, TreeParsing, TreeTextToPath };

const FONTS_DIR: &str = "/usr/share/fonts";
const FB_DEV: &str = "/dev/fb0";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage:\n\tfbsvg <template> [key=value] ...");
        return;
    }

    let data = args[2..].iter().map(|s| s.split_once("=").unwrap())
        .collect::<HashMap<_, _>>();

    let rtree = {
        let mut opt = Options::default();

        opt.resources_dir = fs::canonicalize(&args[1])
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let mut fontdb = fontdb::Database::new();
        fontdb.load_fonts_dir(FONTS_DIR);

        let template = mustache::compile_path(&args[1]).unwrap();
        let svg_data = template.render_to_string(&data).unwrap();

        let mut tree = Tree::from_str(&svg_data, &opt).unwrap();

        tree.convert_text(&fontdb);

        resvg::Tree::from_usvg(&tree)
    };

    let pixmap_size = rtree.size.to_int_size();
    let width = pixmap_size.width();
    let height = pixmap_size.height();
    let mut pixmap = Pixmap::new(width, height).unwrap();

    rtree.render(Transform::default(), &mut pixmap.as_mut());

    let mut fb = Framebuffer::new(FB_DEV).unwrap();
    let frame = pixmap.data_mut();

    for i in (0..frame.len()).step_by(4) {
        frame.swap(i, i + 2);
    }

    fb.write_frame(&frame);
}
