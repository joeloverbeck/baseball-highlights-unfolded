use baseball_highlights_unfolded::{GloveReview, assets::Collection};
use std::{collections::BTreeSet, env, path::Path};

fn main() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let root = args.next().ok_or("Expected one audited asset directory.")?;
    if args.next().is_some() {
        return Err("Expected one audited asset directory.".into());
    }

    let collection = Collection::open(Path::new(&root))?;
    let faces: BTreeSet<_> = collection
        .expert_series_faces()?
        .into_keys()
        .chain(
            collection
                .glove_review_faces(GloveReview::Maris)?
                .into_keys(),
        )
        .collect();
    for face in faces {
        println!("{face}");
    }
    Ok(())
}
