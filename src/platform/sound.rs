use windows::core::PCWSTR;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC, SND_FILENAME, SND_NODEFAULT, SND_PURGE};

use crate::model::SoundKind;

fn alias(name: &str, looping: bool) {
    let data: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut flags = SND_ALIAS | SND_ASYNC | SND_NODEFAULT;
    if looping {
        flags |= windows::Win32::Media::Audio::SND_LOOP;
    }
    unsafe {
        let _ = PlaySoundW(PCWSTR(data.as_ptr()), None, flags);
    }
}

pub fn play(kind: SoundKind, path: &str, looping: bool) {
    match kind {
        SoundKind::Bell => alias("SystemAsterisk", looping),
        SoundKind::Ding => alias("SystemQuestion", looping),
        SoundKind::Chime => alias("SystemExclamation", looping),
        SoundKind::Custom => {
            if path.is_empty() {
                alias("SystemAsterisk", looping);
                return;
            }
            let data: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
            let mut flags = SND_FILENAME | SND_ASYNC | SND_NODEFAULT;
            if looping {
                flags |= windows::Win32::Media::Audio::SND_LOOP;
            }
            unsafe {
                let _ = PlaySoundW(PCWSTR(data.as_ptr()), None, flags);
            }
        }
    }
}

pub fn stop() {
    unsafe {
        let _ = PlaySoundW(PCWSTR::null(), None, SND_PURGE);
    }
}
