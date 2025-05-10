use std::{
    env, fs,
    io::{self, Read},
    os::unix::fs::MetadataExt,
    path::Path,
};

use nu_ansi_term::Style;
use chrono::{DateTime, Local};
use lscolors::{LsColors, Style as LsStyle};
use users::{get_group_by_gid, get_user_by_uid};
use std::os::unix::fs::PermissionsExt;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let target = if args.len() > 1 { &args[1] } else { "." };

    let path = Path::new(target);
    if path.is_file() {
        cat_file(path)?;
    } else {
        list_dir(path)?;
    }
    Ok(())
}

fn cat_file(path: &Path) -> io::Result<()> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    print!("{}", contents);
    Ok(())
}

fn display_permissions(metadata: &fs::Metadata) -> String {
    let perms = metadata.permissions();
    let mode = perms.mode(); // Unix-specific: std::os::unix::fs::PermissionsExt
    let file_type = if metadata.is_dir() { 'd' } else { '-' };

    format!(
        "{}{}{}{}{}{}{}{}{}{}",
        file_type,
        if mode & 0o400 != 0 { 'r' } else { '-' },
        if mode & 0o200 != 0 { 'w' } else { '-' },
        if mode & 0o100 != 0 { 'x' } else { '-' },
        if mode & 0o040 != 0 { 'r' } else { '-' },
        if mode & 0o020 != 0 { 'w' } else { '-' },
        if mode & 0o010 != 0 { 'x' } else { '-' },
        if mode & 0o004 != 0 { 'r' } else { '-' },
        if mode & 0o002 != 0 { 'w' } else { '-' },
        if mode & 0o001 != 0 { 'x' } else { '-' },
    )
}


fn list_dir(path: &Path) -> io::Result<()> {
    let lscolors = LsColors::from_env().unwrap_or_default();
    let entries: Vec<_> = fs::read_dir(path)?.filter_map(Result::ok).collect();

    for entry in entries {
        let metadata = entry.metadata()?;
        let file_type = if metadata.is_dir() {
            "d"
        } else if metadata.is_symlink() {
            "l"
        } else {
            "-"
        };

        let permissions_str = display_permissions(&metadata);
        let mode = metadata.mode();
        let perms = format!(
            "{}{}{}{}{}{}{}{}{}",
            if mode & 0o400 != 0 { "r" } else { "-" },
            if mode & 0o200 != 0 { "w" } else { "-" },
            if mode & 0o100 != 0 { "x" } else { "-" },
            if mode & 0o040 != 0 { "r" } else { "-" },
            if mode & 0o020 != 0 { "w" } else { "-" },
            if mode & 0o010 != 0 { "x" } else { "-" },
            if mode & 0o004 != 0 { "r" } else { "-" },
            if mode & 0o002 != 0 { "w" } else { "-" },
            if mode & 0o001 != 0 { "x" } else { "-" },
        );

        let nlink = metadata.nlink();
        let uid = metadata.uid();
        let gid = metadata.gid();

        let user = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or(uid.to_string());
        let group = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or(gid.to_string());

        let size = metadata.size();
        let mtime: DateTime<Local> = DateTime::from(metadata.modified()?);
        let formatted_time = mtime.format("%b %d %H:%M");

        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        let style = lscolors.style_for_path(&entry.path());

        let colored_name = match style {
    Some(LsStyle { foreground, background, font_style, .. }) => {
        let mut ansi = nu_ansi_term::Style::new();
        if let Some(fg) = foreground {
            ansi = ansi.fg(fg.to_nu_ansi_term_color());
        }
        if let Some(bg) = background {
            ansi = ansi.on(bg.to_nu_ansi_term_color());
        }
        if font_style.bold {
            ansi = ansi.bold();
        }
        ansi.paint(file_name_str.to_string())
    }
    None => Style::new().paint(file_name_str.to_string()),
};


        println!(
            "{}{} {:>2} {:<8} {:<8} {:>8} {} {}",
            file_type, perms, nlink, user, group, size, formatted_time, colored_name
        );
    }
    Ok(())
}
