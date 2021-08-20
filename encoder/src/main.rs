use encoder_lib::{Encoder, Params, Source, Stats};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn main() {
    let source = Source::from_dir("../video").expect("Couldn't load video");
    let images = source.images().count();
    let params_sets = prepare_params_sets();
    let results = perform_encodings(source, params_sets);

    let (params, stats, code) = {
        results
            .into_iter()
            .max_by(|(_, a, _), (_, b, _)| {
                let a = a.frames;
                let b = b.frames;

                a.cmp(&b)
            })
            .unwrap()
    };

    eprintln!("{:#?}", params);
    eprintln!("{:#?}", stats);
    eprintln!("(encoded {} frames out of {})", stats.frames, images);
    println!("{}", code);
}

fn prepare_params_sets() -> Vec<Params> {
    let video_width = 84;
    let video_height = 48;
    let max_code_size = 30050;

    let mut params_sets = Vec::new();

    for block_width in 1..video_width {
        for block_height in 1..video_height {
            let params = Params::new(
                block_width,
                block_height,
                video_width,
                video_height,
                max_code_size,
            );

            if let Ok(params) = params {
                params_sets.push(params);
            }
        }
    }

    params_sets
}

fn perform_encodings(source: Source, params_sets: Vec<Params>) -> Vec<(Params, Stats, String)> {
    let len = params_sets.len();

    params_sets
        .into_par_iter()
        .progress_count(len as u64)
        .map(move |params| perform_encoding(&source, params))
        .collect()
}

fn perform_encoding(source: &Source, params: Params) -> (Params, Stats, String) {
    let mut encoder = Encoder::new(&params).expect("Couldn't create encoder");
    let mut images = source.images();

    while let Some(image) = images.next() {
        if !encoder.add(image) {
            break;
        }
    }

    let (stats, code) = encoder.finish();

    (params, stats, code)
}
