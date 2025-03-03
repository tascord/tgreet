use std::{collections::VecDeque, env, fs, io::BufRead, process::Command};

use anyhow::{anyhow, bail};
use mpris::PlayerFinder;
use owo_colors::OwoColorize;

fn main() {
    let img = image().unwrap_or("/home/flora/Downloads/moon.jpg".to_string());
    let lines = Command::new("catimg")
        .args(["-w", "40", &img])
        .output()
        .unwrap()
        .stdout
        .lines()
        .map(|l| l.unwrap().to_string())
        .collect::<Vec<_>>();

    let mut info = VecDeque::from_iter(info().into_iter());
    let padding = lines.len().saturating_sub(info.len());
    if padding > 1 {
        let half_pad = padding / 2;
        for _ in 0..half_pad {
            info.push_back(String::new());
            info.push_front(String::new());
        }
    }

    lines.into_iter().zip(info.into_iter()).for_each(|(a, b)| {
        println!("{a}\t{}", b.trim());
    })
}

fn info() -> Vec<String> {
    let mut block = Vec::new();
    macro_rules! triad {
        ($pat: literal, $v: expr) => {
            match $v {
                Ok(v) => block.push(format!($pat, v)),
                _ => {}
            }
        };
    }

    triad!("Hello, {}", env::var("USER"));
    triad!(
        "It is {}",
        Command::new("timedatectl")
            .args(["show", "-P", "TimeUSec"])
            .output()
            .map(|o| String::from_utf8(o.stdout).ok())
            .ok()
            .flatten()
            .ok_or("")
    );

    if let Ok((track, album)) = current_song() {
        block.push(format!("{} - {}", track.bold(), album.dimmed()));
    } else {
        triad!(
            "{}",
            Command::new("misfortune")
                .arg("-as")
                .arg("-n 60")
                .output()
                .map(|o| String::from_utf8(o.stdout)
                    .map(|v| format!(
                        "\"{}\"",
                        v.replace("\n", " ").replace("  ", " ").trim().to_string()
                    )
                    .italic()
                    .dimmed()
                    .to_string())
                    .ok())
                .ok()
                .flatten()
                .ok_or("")
        );
    }

    let term = env::var("TERM").ok();
    let shell = env::var("SHELL")
        .map(|v| v.split('/').last().map(|v| v.to_string()))
        .ok()
        .flatten();

    let str = vec![term, shell]
        .into_iter()
        .filter_map(|v| v)
        .map(|v| v.dimmed().to_string())
        .collect::<Vec<_>>();

    if !str.is_empty() {
        block.push(String::new());
        block.push(str.join(" | "));
    }

    block
}

fn current_song() -> anyhow::Result<(String, String)> {
    let metadata = PlayerFinder::new()?.find_active()?.get_metadata()?;
    if metadata.album_name().map(|v| v.is_empty()).unwrap_or(false) {
        bail!("")
    }

    Ok((
        metadata.title().map(|v| v.to_string()).ok_or(anyhow!(""))?,
        metadata
            .album_name()
            .map(|v| v.to_string())
            .ok_or(anyhow!(""))?,
    ))
}

fn image() -> anyhow::Result<String> {
    let metadata = PlayerFinder::new()?.find_active()?.get_metadata()?;
    if metadata.album_name().map(|v| v.is_empty()).unwrap_or(false) {
        bail!("")
    }

    let path = format!(
        "/tmp/album-{}",
        metadata
            .album_name()
            .ok_or(anyhow!(""))?
            .chars()
            .map(|c| match c.is_alphanumeric() {
                true => c,
                false => '_',
            })
            .collect::<String>()
    );

    if fs::exists(path.to_string())? {
        return Ok(path);
    }

    Command::new("wget")
        .args([metadata.art_url().ok_or(anyhow!(""))?, "-O", &path, "-q"])
        .status()?
        .success()
        .then_some(())
        .ok_or(anyhow!(""))?;

    Ok(path)
}
