use clap::Parser;
use human_repr::HumanDuration;
use human_repr::HumanDurationData;
use std::fmt;
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// format of currently playing audio
    #[arg(short, long, default_value = "%t - %a")]
    format: String,
}

struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub position: HumanDurationData,
}

impl fmt::Display for MediaInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.position != "0ns" && !self.artist.is_empty() { // normal format
            write!(f, "{} - {} ({})", self.title, self.artist, self.position)
        } else if !self.artist.is_empty() && self.position == "0ns" { // position empty
            write!(f, "{} - {}", self.title, self.artist)
        } else if self.position != "0ns" && self.artist.is_empty() { // artist empty
            write!(f, "{} ({})", self.title, self.position)
        } else {
            write!(f, "{}", self.title)
        }
    }
}

impl MediaInfo {
    fn display(&self, f: String) -> String {
        f
            .replace("%t", &self.title)
            .replace("%a", &self.artist)
            .replace("%p", &format!("{}", &self.position))
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let args = Args::parse();

    let playing = match get_media_info().await {
        Ok(song) => song,
        Err(_) => MediaInfo {
            title: "No Song Playing".to_owned(),
            artist: "".to_owned(),
            position: 0.human_duration(),
        }, // No media playing
    };
    println!("{}", playing.display(args.format));
    Ok(())
}

async fn get_media_info() -> Result<MediaInfo, windows::core::Error> {
    let mp = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
        // Gets the async TransportControlsSessionManager so that we can work with it
        Ok(stuff) => match stuff.await {
            Ok(more_stuff) => more_stuff,
            Err(err) => return Err(err),
        },
        Err(err) => return Err(err),
    };
    let current_session = match mp.GetCurrentSession() {
        // Gets current media player
        Ok(stuff) => stuff,
        Err(err) => return Err(err),
    };
    let timeline = match current_session.GetTimelineProperties() {
        // Gets current media player
        Ok(stuff) => stuff,
        Err(err) => return Err(err),
    };
    let info = match current_session.TryGetMediaPropertiesAsync() {
        // Get media properties
        Ok(stuff) => match stuff.await {
            Ok(stuf) => stuf,
            Err(err) => return Err(err),
        },
        Err(err) => return Err(err),
    };
    let title = match info.Title() {
        Ok(stuff) => stuff,
        Err(err) => return Err(err),
    };
    let artist = match info.Artist() {
        Ok(stuff) => stuff,
        Err(err) => return Err(err),
    };
    let position = match timeline.Position() {
        Ok(stuff) => stuff,
        Err(err) => return Err(err),
    };
    let base: i64 = 10;
    // Return song title
    Ok(MediaInfo {
        title: title.to_string(),
        artist: artist.to_string(),
        position: ((position.Duration / base.pow(7)).human_duration()),
    })
}
