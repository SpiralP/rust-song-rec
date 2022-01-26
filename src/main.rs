mod fingerprinting {
    pub mod algorithm;
    pub mod communication;
    mod hanning;
    pub mod signature_format;
    mod user_agent;
}

mod core {
    pub mod http_thread;
}

use crate::{core::http_thread::try_recognize_song, fingerprinting::algorithm::SignatureGenerator};

fn main() {
    let signature = SignatureGenerator::make_signature_from_file("file.wav").unwrap();
    // println!("{:#?}", signature);

    let result = try_recognize_song(signature).unwrap();
    println!("{:#?}", result.song_name);
    println!("{:#?}", result.album_name);
    println!("{:#?}", result.artist_name);
    println!("{:#?}", result.track_key);
}
