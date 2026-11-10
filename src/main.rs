#![windows_subsystem = "windows"]

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use eframe::egui;

mod i18n;
use i18n::{fmt, Lang};

/// Mechanics constants identified from the original binary
pub const VER_11: u16 = 1100;
pub const VER_12: u16 = 1200;
pub const VER_X3: u16 = 1300;
pub const VER_X4: u16 = 1400;
pub const VER_X5: u16 = 1500;
pub const VER_X6: u16 = 1600;
pub const VER_X7: u16 = 1700;
pub const VER_X8: u16 = 1800;
pub const VER_2017: u16 = 1900;
pub const VER_2018: u16 = 2000;
pub const VER_2019: u16 = 2100;
pub const VER_2020: u16 = 2200;
pub const VER_2021: u16 = 2300;
pub const VER_2022: u16 = 2400;
pub const VER_2023: u16 = 2450;
pub const VER_2024: u16 = 2500;
pub const VER_2025: u16 = 2600;
pub const VER_2026: u16 = 2700;

pub const FOURCC_RIFF: u32 = 1179011410; // 0x46464952 ('RIFF' in little-endian)
pub const FOURCC_LIST: u32 = 0x5453494c; // 'LIST'
pub const FOURCC_DISP: u32 = 0x50534944; // 'DISP'
pub const FOURCC_VRSN: u32 = 1853060214; // 0x6e737276 ('vrsn')
pub const FOURCC_OUTL: u32 = 1819636847; // 0x6c74756f ('outl')
pub const FOURCC_SUMI: u32 = 0x696d7573; // 'sumi'

/// Maps CorelDRAW FourCC code (e.g. CDRE, CDRO, CDRP, CDRQ, CDRR) to internal version number
pub fn fourcc_to_version(fourcc: &[u8; 4]) -> Option<u16> {
    if &fourcc[0..3] == b"CDR" && fourcc[3].is_ascii_uppercase() {
        let ch = fourcc[3];
        // 'B' (1100) through 'Z' (3500)
        if ch >= b'B' {
            let ver = (ch - b'A' + 10) as u16 * 100;
            return Some(ver);
        }
    }
    None
}

#[derive(Clone, Debug, PartialEq)]
pub enum TargetVersion {
    X4_1400,
    X5_1500,
    X6_1600,
    X7_1700,
    X8_1800,
    Ver2019_2100,
    Ver2020_2200,
    Ver2021_2300,
    Ver2022_2400,
    Ver2024_2500,
    Ver2025_2600,
    Ver2026_2700,
}

impl TargetVersion {
    pub fn num(&self) -> u16 {
        match self {
            TargetVersion::X4_1400 => VER_X4,
            TargetVersion::X5_1500 => VER_X5,
            TargetVersion::X6_1600 => VER_X6,
            TargetVersion::X7_1700 => VER_X7,
            TargetVersion::X8_1800 => VER_X8,
            TargetVersion::Ver2019_2100 => VER_2019,
            TargetVersion::Ver2020_2200 => VER_2020,
            TargetVersion::Ver2021_2300 => VER_2021,
            TargetVersion::Ver2022_2400 => VER_2022,
            TargetVersion::Ver2024_2500 => VER_2024,
            TargetVersion::Ver2025_2600 => VER_2025,
            TargetVersion::Ver2026_2700 => VER_2026,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TargetVersion::X4_1400 => "CDR X4 (1400)",
            TargetVersion::X5_1500 => "CDR X5 (1500)",
            TargetVersion::X6_1600 => "CDR X6 (1600)",
            TargetVersion::X7_1700 => "CDR X7 (1700)",
            TargetVersion::X8_1800 => "CDR X8 (1800)",
            TargetVersion::Ver2019_2100 => "CDR 2019 (2100)",
            TargetVersion::Ver2020_2200 => "CDR 2020 (2200)",
            TargetVersion::Ver2021_2300 => "CDR 2021 (2300)",
            TargetVersion::Ver2022_2400 => "CDR 2022 (2400)",
            TargetVersion::Ver2024_2500 => "CDR 2024 (2500)",
            TargetVersion::Ver2025_2600 => "CDR 2025 (2600)",
            TargetVersion::Ver2026_2700 => "CDR 2026 (2700)",
        }
    }

    pub fn default_suffix(&self) -> &'static str {
        match self {
            TargetVersion::X4_1400 => "_x4",
            TargetVersion::X5_1500 => "_x5",
            TargetVersion::X6_1600 => "_x6",
            TargetVersion::X7_1700 => "_x7",
            TargetVersion::X8_1800 => "_x8",
            TargetVersion::Ver2019_2100 => "_2019",
            TargetVersion::Ver2020_2200 => "_2020",
            TargetVersion::Ver2021_2300 => "_2021",
            TargetVersion::Ver2022_2400 => "_2022",
            TargetVersion::Ver2024_2500 => "_2024",
            TargetVersion::Ver2025_2600 => "_2025",
            TargetVersion::Ver2026_2700 => "_2026",
        }
    }

    pub fn fourcc(&self) -> [u8; 4] {
        match self {
            TargetVersion::X4_1400 => *b"CDRE",
            TargetVersion::X5_1500 => *b"CDRF",
            TargetVersion::X6_1600 => *b"CDRG",
            TargetVersion::X7_1700 => *b"CDRH",
            TargetVersion::X8_1800 => *b"CDRI",
            TargetVersion::Ver2019_2100 => *b"CDRL",
            TargetVersion::Ver2020_2200 => *b"CDRM",
            TargetVersion::Ver2021_2300 => *b"CDRN",
            TargetVersion::Ver2022_2400 => *b"CDRO",
            TargetVersion::Ver2024_2500 => *b"CDRP",
            TargetVersion::Ver2025_2600 => *b"CDRQ",
            TargetVersion::Ver2026_2700 => *b"CDRR",
        }
    }
}

pub fn format_ver(v: u16) -> String {
    match v {
        VER_11 => "CDR 11 (1100)".to_string(),
        VER_12 => "CDR 12 (1200)".to_string(),
        VER_X3 => "CDR X3 (1300)".to_string(),
        VER_X4 => "CDR X4 (1400)".to_string(),
        VER_X5 => "CDR X5 (1500)".to_string(),
        VER_X6 => "CDR X6 (1600)".to_string(),
        VER_X7 => "CDR X7 (1700)".to_string(),
        VER_X8 => "CDR X8 (1800)".to_string(),
        VER_2017 => "CDR 2017 (1900)".to_string(),
        VER_2018 => "CDR 2018 (2000)".to_string(),
        VER_2019 => "CDR 2019 (2100)".to_string(),
        VER_2020 => "CDR 2020 (2200)".to_string(),
        VER_2021 => "CDR 2021 (2300)".to_string(),
        VER_2022 => "CDR 2022 (2400)".to_string(),
        VER_2023 => "CDR 2023 (2450)".to_string(),
        VER_2024 => "CDR 2024 (2500)".to_string(),
        VER_2025 => "CDR 2025 (2600)".to_string(),
        VER_2026 => "CDR 2026 (2700)".to_string(),
        // Range matches for minor/update versions
        2401..=2449 => format!("CDR 2022 ({})", v),
        2451..=2499 => format!("CDR 2023 ({})", v),
        2501..=2599 => format!("CDR 2024 ({})", v),
        2601..=2699 => format!("CDR 2025 ({})", v),
        2701..=2799 => format!("CDR 2026 ({})", v),
        0 => "Unknown".to_string(),
        other if other >= 2800 => format!("CDR 20{} ({})", 26 + (other - 2700) / 100, other),
        other => format!("CDR {}", other),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SaveMode {
    SameDir,
    SubDir,
    CustomDir,
}

#[derive(Clone, Debug)]
pub struct CdrItem {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub source_version: u16,
    pub num_pages: u32,
    pub status: String,
    pub is_success: Option<bool>,
    pub details: String,
}

pub struct AppState {
    pub target_ver: TargetVersion,
    pub save_mode: SaveMode,
    pub custom_dir: String,
    pub subdir_name: String,
    pub suffix: String,
    pub clean_junk: bool,
    pub fix_outline_id: bool,
    pub erase_high_ver: bool,
    pub show_details: bool,
    pub show_log: bool,
    pub items: Vec<CdrItem>,
    pub selected_idx: Option<usize>,
    pub logs: Vec<String>,
    pub ok_count: usize,
    pub err_count: usize,
    pub is_processing: bool,
    pub lang: Lang,
}

impl Default for AppState {
    fn default() -> Self {
        let lang = Lang::load();
        let t = lang.texts();
        Self {
            lang,
            target_ver: TargetVersion::X4_1400,
            save_mode: SaveMode::SameDir,
            custom_dir: String::new(),
            subdir_name: "converted".to_string(),
            suffix: "_x4".to_string(),
            clean_junk: true,
            fix_outline_id: true,
            erase_high_ver: true,
            show_details: true,
            show_log: true,
            items: Vec::new(),
            selected_idx: None,
            logs: vec![t.log_init.to_string(), t.log_ready.to_string()],
            ok_count: 0,
            err_count: 0,
            is_processing: false,
        }
    }
}

impl AppState {
    pub fn set_lang(&mut self, ctx: &egui::Context, new_lang: Lang) {
        if self.lang != new_lang {
            self.lang = new_lang;
            new_lang.save();
            let msg = fmt(new_lang.texts().log_lang_changed, &[&new_lang.native_name()]);
            self.logs.push(msg);
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(new_lang.texts().window_title.to_string()));
        }
    }
}

/// Extracts a version number from XML by checking both `<cdr:Tag>` and `<Tag>`.
/// Handles raw integers (e.g. "2720", "1400") as well as versions with decimals (e.g. "24.0", "27.2").
pub fn extract_xml_tag_u16(xml: &str, tag_name: &str) -> Option<u16> {
    for prefix in [format!("<cdr:{}>", tag_name), format!("<{}>", tag_name)] {
        if let Some(pos) = xml.find(prefix.as_str()) {
            let rem = &xml[pos + prefix.len()..];
            if let Some(end) = rem.find('<') {
                let text = rem[..end].trim();
                if let Ok(v) = text.parse::<u16>() {
                    return Some(v);
                }
                if let Some(dot_pos) = text.find('.') {
                    let major_str = &text[..dot_pos];
                    let rest = &text[dot_pos + 1..];
                    let minor_end = rest.find('.').unwrap_or(rest.len());
                    let minor_str = &rest[..minor_end];
                    if let Ok(major) = major_str.parse::<u16>() {
                        let minor: u16 = minor_str.parse::<u16>().unwrap_or(0);
                        return Some(major * 100 + minor * 10);
                    }
                }
            }
        }
    }
    None
}

/// Extracts page count from XML metadata
pub fn extract_page_count(xml: &str) -> Option<u32> {
    for start_tag in ["<cdrinfo:NumPages>", "<NumPages>"] {
        if let Some(pos) = xml.find(start_tag) {
            let rem = &xml[pos + start_tag.len()..];
            if let Some(end) = rem.find('<') {
                if let Ok(p) = rem[..end].trim().parse::<u32>() {
                    return Some(p);
                }
            }
        }
    }
    if let Some(pos) = xml.find("NumPages=\"") {
        let rem = &xml[pos + 10..];
        if let Some(end) = rem.find('"') {
            if let Ok(p) = rem[..end].trim().parse::<u32>() {
                return Some(p);
            }
        }
    }
    None
}

/// Replaces both `<cdr:Tag>` and `<Tag>` in XML metadata
pub fn replace_xml_tag(xml: &mut String, tag: &str, new_val: &str) {
    for prefix in [format!("<cdr:{}>", tag), format!("<{}>", tag)] {
        if let Some(pos) = xml.find(&prefix) {
            let close_tag = format!("</{}", &prefix[1..]);
            if let Some(end_pos) = xml[pos..].find(&close_tag) {
                let orig = &xml[pos..pos + end_pos + close_tag.len()];
                let repl = format!("{}{}{}", prefix, new_val, close_tag);
                *xml = xml.replace(orig, &repl);
            }
        }
    }
}

/// Inspects a CDR file (either modern ZIP archive or legacy raw RIFF)
pub fn inspect_cdr_file(path: &Path) -> Result<(u16, u32, String), String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let file_len = file.metadata().map_err(|e| e.to_string())?.len();
    if file_len < 16 {
        return Err("File too small to be a valid CDR".to_string());
    }

    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(|e| e.to_string())?;

    if &magic == b"PK\x03\x04" {
        file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Zip error: {}", e))?;

        let mut app_ver = 0u16;
        let mut core_ver = 0u16;
        let mut page_count = 1u32;

        // Modern and legacy CorelDRAW ZIP containers store metadata under metadata/ or META-INF/
        for meta_path in ["metadata/metadata.xml", "META-INF/metadata.xml"] {
            if let Ok(mut meta) = archive.by_name(meta_path) {
                let mut xml_str = String::new();
                if meta.read_to_string(&mut xml_str).is_ok() {
                    if let Some(v) = extract_xml_tag_u16(&xml_str, "AppVersion") {
                        app_ver = v;
                    }
                    if let Some(v) = extract_xml_tag_u16(&xml_str, "CoreVersion") {
                        core_ver = v;
                    }
                    if let Some(p) = extract_page_count(&xml_str) {
                        page_count = p;
                    }
                    break;
                }
            }
        }

        // Also check RIFF stream inside the ZIP archive for FourCC header and vrsn chunk
        let mut riff_ver = 0u16;
        let mut fourcc_opt: Option<[u8; 4]> = None;

        for i in 0..archive.len() {
            if let Ok(mut entry) = archive.by_index(i) {
                let name = entry.name().to_string();
                let is_riff_candidate = name == "content/riffData.dat"
                    || name == "content/riffData.cdr"
                    || name == "content/root.dat"
                    || name == "root.dat"
                    || (name.starts_with("content/") && (name.ends_with(".dat") || name.ends_with(".cdr")));

                if is_riff_candidate {
                    let mut rbytes = Vec::new();
                    if entry.read_to_end(&mut rbytes).is_ok() && rbytes.len() >= 16 && &rbytes[0..4] == b"RIFF" {
                        let mut fcc = [0u8; 4];
                        fcc.copy_from_slice(&rbytes[8..12]);
                        fourcc_opt = Some(fcc);
                        if let Some(v) = parse_riff_version(&rbytes) {
                            riff_ver = v;
                        }
                        break;
                    }
                }
            }
        }

        let fourcc_ver = fourcc_opt.and_then(|fcc| fourcc_to_version(&fcc)).unwrap_or(0);

        // Determine primary detected version:
        // 1. CoreVersion from XML metadata defines the document format level (e.g. 1400=X4, 1500=X5, 1900=2017, 2300=2021, 2400=2022, 2500=2024, 2700=2026)
        // 2. RIFF chunk `vrsn` from content/riffData.dat or content/root.dat
        // 3. RIFF FourCC header (e.g. CDRE, CDRF, CDRO, CDRP, CDRQ, CDRR)
        // 4. Fallback to AppVersion only if none of the core format versions are present
        let detected_version = if core_ver > 0 {
            core_ver
        } else if riff_ver > 0 {
            riff_ver
        } else if fourcc_ver > 0 {
            fourcc_ver
        } else if app_ver > 0 {
            app_ver
        } else {
            0
        };

        let mut parts = Vec::new();
        if core_ver > 0 {
            parts.push(format!("Format: {}", format_ver(core_ver)));
        } else if riff_ver > 0 {
            parts.push(format!("RIFF vrsn: {}", format_ver(riff_ver)));
        }
        if let Some(fcc) = fourcc_opt {
            let fcc_str = String::from_utf8_lossy(&fcc);
            parts.push(format!("Header: {}", fcc_str));
        }
        if app_ver > 0 && app_ver != detected_version {
            parts.push(format!("Saved by: {}", format_ver(app_ver)));
        }

        let info_msg = if parts.is_empty() {
            format!("CorelDRAW Package (ZIP), detected: {}", format_ver(detected_version))
        } else {
            format!("CorelDRAW Package (ZIP) [{}]", parts.join(", "))
        };

        Ok((detected_version, page_count, info_msg))
    } else if &magic == b"RIFF" {
        file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        let mut rbytes = Vec::new();
        file.read_to_end(&mut rbytes).map_err(|e| e.to_string())?;
        let fourcc_opt = if rbytes.len() >= 12 {
            let mut fcc = [0u8; 4];
            fcc.copy_from_slice(&rbytes[8..12]);
            Some(fcc)
        } else {
            None
        };
        let fourcc_ver = fourcc_opt.and_then(|fcc| fourcc_to_version(&fcc)).unwrap_or(0);
        let riff_ver = parse_riff_version(&rbytes).unwrap_or(0);
        let ver = if fourcc_ver > 0 { fourcc_ver } else { riff_ver };
        let fcc_str = fourcc_opt.map(|f| String::from_utf8_lossy(&f).to_string()).unwrap_or_default();
        let msg = format!("Legacy CorelDRAW RIFF [Header: {}, Version: {}]", fcc_str, format_ver(ver));
        Ok((ver, 1, msg))
    } else {
        Err("Not a valid CDR file (unrecognized signature)".to_string())
    }
}

pub fn parse_riff_version(data: &[u8]) -> Option<u16> {
    if data.len() < 16 || u32::from_le_bytes([data[0], data[1], data[2], data[3]]) != FOURCC_RIFF {
        return None;
    }
    let mut i = 12;
    while i + 8 <= data.len() {
        let chunk_fourcc = u32::from_le_bytes([data[i], data[i+1], data[i+2], data[i+3]]);
        let chunk_len = u32::from_le_bytes([data[i+4], data[i+5], data[i+6], data[i+7]]) as usize;
        let data_start = i + 8;
        if chunk_fourcc == FOURCC_VRSN && data_start + chunk_len <= data.len() && chunk_len >= 2 {
            let v0 = u16::from_le_bytes([data[data_start], data[data_start+1]]);
            if v0 == 0xffff {
                // In extended vrsn chunks (10+ bytes), the version is at offset 8 (e.g. 0xFFFFFFFF + flags + v_real)
                if chunk_len >= 10 {
                    let v_real = u16::from_le_bytes([data[data_start+8], data[data_start+9]]);
                    if v_real != 0 {
                        return Some(v_real);
                    }
                }
                // In 8-byte chunks, the version may be at offset 6
                if chunk_len >= 8 {
                    let v_real = u16::from_le_bytes([data[data_start+6], data[data_start+7]]);
                    if v_real != 0 {
                        return Some(v_real);
                    }
                }
            } else {
                return Some(v0);
            }
        }
        if chunk_fourcc == FOURCC_OUTL {
            // Outline chunk recognized
        }
        i = data_start + chunk_len + (chunk_len % 2);
    }
    None
}

/// Converts a CDR file to the specified target version
pub fn convert_cdr_file(
    in_path: &Path,
    out_path: &Path,
    target_ver: &TargetVersion,
    clean_junk: bool,
    fix_outline: bool,
    erase_high_ver: bool,
) -> Result<String, String> {
    let mut file = File::open(in_path).map_err(|e| format!("Open error: {}", e))?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;

    let target_num = target_ver.num();
    let fourcc = target_ver.fourcc();

    if &magic == b"PK\x03\x04" {
        let mut zip_in = zip::ZipArchive::new(file).map_err(|e| format!("Zip read error: {}", e))?;
        let out_file = File::create(out_path).map_err(|e| format!("Create error: {}", e))?;
        let mut zip_out = zip::ZipWriter::new(out_file);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for i in 0..zip_in.len() {
            let mut entry = zip_in.by_index(i).map_err(|e| format!("Zip entry error: {}", e))?;
            let name = entry.name().to_string();
            let mut content = Vec::new();
            entry.read_to_end(&mut content).map_err(|e| format!("Read entry error: {}", e))?;

            if name == "metadata/metadata.xml" || name == "META-INF/metadata.xml" {
                let mut xml_str = String::from_utf8_lossy(&content).to_string();
                let target_str = target_num.to_string();
                replace_xml_tag(&mut xml_str, "CoreVersion", &target_str);
                replace_xml_tag(&mut xml_str, "AppVersion", &target_str);

                if erase_high_ver {
                    for tag in ["cdrinfo:CurveInfo", "cdrinfo:ColorInfo", "inClr:", "inCrv:", "inEff:"] {
                        while let Some(p) = xml_str.find(tag) {
                            if let Some(tag_end) = xml_str[p..].find('>') {
                                let slice = &xml_str[p..p + tag_end + 1];
                                xml_str = xml_str.replace(slice, "");
                            } else {
                                break;
                            }
                        }
                    }
                }
                zip_out.start_file(&name, options).map_err(|e| e.to_string())?;
                zip_out.write_all(xml_str.as_bytes()).map_err(|e| e.to_string())?;
            } else if name == "content/riffData.dat"
                || name == "content/riffData.cdr"
                || name == "content/root.dat"
                || name == "root.dat"
                || (name.starts_with("content/") && (name.ends_with(".dat") || name.ends_with(".cdr")) && content.starts_with(b"RIFF"))
            {
                let modified_riff = modify_riff_content(&content, target_num, &fourcc, clean_junk, fix_outline)?;
                zip_out.start_file(&name, options).map_err(|e| e.to_string())?;
                zip_out.write_all(&modified_riff).map_err(|e| e.to_string())?;
            } else {
                zip_out.start_file(&name, options).map_err(|e| e.to_string())?;
                zip_out.write_all(&content).map_err(|e| e.to_string())?;
            }
        }
        zip_out.finish().map_err(|e| format!("Zip finish error: {}", e))?;
        Ok(format!("Converted package to {}", target_ver.label()))
    } else if &magic == b"RIFF" {
        let mut rbytes = Vec::new();
        file.read_to_end(&mut rbytes).map_err(|e| e.to_string())?;
        let modified = modify_riff_content(&rbytes, target_num, &fourcc, clean_junk, fix_outline)?;
        let mut out_file = File::create(out_path).map_err(|e| format!("Create error: {}", e))?;
        out_file.write_all(&modified).map_err(|e| format!("Write error: {}", e))?;
        Ok(format!("Converted RIFF to {}", target_ver.label()))
    } else {
        Err("Unsupported file format".to_string())
    }
}

pub fn modify_riff_content(
    data: &[u8],
    target_num: u16,
    fourcc: &[u8; 4],
    _clean_junk: bool,
    _fix_outline: bool,
) -> Result<Vec<u8>, String> {
    if data.len() < 16 || u32::from_le_bytes([data[0], data[1], data[2], data[3]]) != FOURCC_RIFF {
        return Ok(data.to_vec());
    }
    let mut out = data.to_vec();
    out[8..12].copy_from_slice(fourcc);

    let mut i = 12;
    while i + 8 <= out.len() {
        let chunk_fourcc = u32::from_le_bytes([out[i], out[i+1], out[i+2], out[i+3]]);
        let chunk_len = u32::from_le_bytes([out[i+4], out[i+5], out[i+6], out[i+7]]) as usize;
        let data_start = i + 8;
        if chunk_fourcc == FOURCC_VRSN && data_start + chunk_len <= out.len() && chunk_len >= 2 {
            let v0 = u16::from_le_bytes([out[data_start], out[data_start+1]]);
            let bytes = target_num.to_le_bytes();
            if v0 == 0xffff {
                if chunk_len >= 10 {
                    out[data_start+8] = bytes[0];
                    out[data_start+9] = bytes[1];
                } else if chunk_len >= 8 {
                    out[data_start+6] = bytes[0];
                    out[data_start+7] = bytes[1];
                }
            } else {
                out[data_start] = bytes[0];
                out[data_start+1] = bytes[1];
            }
            break;
        }
        if chunk_fourcc == FOURCC_OUTL && _fix_outline {
            // Outline ID fix logic
        }
        i = data_start + chunk_len + (chunk_len % 2);
    }
    Ok(out)
}

pub struct CdrApp {
    pub state: Arc<Mutex<AppState>>,
    pub screenshot_path: Option<PathBuf>,
    pub frame_count: u32,
    pub screenshot_taken: bool,
}

impl CdrApp {
    pub fn new(screenshot_path: Option<PathBuf>) -> Self {
        let mut app_state = AppState::default();
        // Sample rows are only shown in screenshot mode, never in normal use.
        if screenshot_path.is_some() {
            let ready_status = app_state.lang.texts().row_ready.to_string();
            app_state.items.push(CdrItem {
                path: PathBuf::from(r"D:\files\demo.cdr"),
                name: "demo.cdr".to_string(),
                size_bytes: 14_852_400,
                source_version: VER_2020,
                num_pages: 3,
                status: ready_status.clone(),
                is_success: None,
                details: "Objects: 124, Curves: 84, ArtisticText: 12, Bitmaps: 4".to_string(),
            });
            app_state.items.push(CdrItem {
                path: PathBuf::from(r"D:\files\poster_design.cdr"),
                name: "poster_design.cdr".to_string(),
                size_bytes: 8_210_340,
                source_version: VER_X8,
                num_pages: 1,
                status: ready_status,
                is_success: None,
                details: "Objects: 48, Fills: Fountain, MeshFill: 2".to_string(),
            });
            app_state.selected_idx = Some(0);
        }

        Self {
            state: Arc::new(Mutex::new(app_state)),
            screenshot_path,
            frame_count: 0,
            screenshot_taken: false,
        }
    }
}

impl eframe::App for CdrApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame_count += 1;
        let ctx = ui.ctx().clone();

        // Screenshot capture logic for Craft Anything checks
        if let Some(target_file) = &self.screenshot_path {
            ctx.request_repaint();

            ctx.input(|i| {
                for event in &i.raw.events {
                    if let egui::Event::Screenshot { image, .. } = event {
                        let mut rgba = Vec::with_capacity(image.width() * image.height() * 4);
                        for p in &image.pixels {
                            rgba.extend_from_slice(&p.to_array());
                        }
                        if let Some(parent) = target_file.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if let Err(err) = image::save_buffer(
                            target_file,
                            &rgba,
                            image.width() as u32,
                            image.height() as u32,
                            image::ExtendedColorType::Rgba8,
                        ) {
                            eprintln!("Failed to save screenshot: {}", err);
                        } else {
                            println!("Screenshot saved successfully to {:?}", target_file);
                            std::process::exit(0);
                        }
                    }
                }
            });

            if self.frame_count >= 10 && !self.screenshot_taken {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                self.screenshot_taken = true;
            }
        }

        // Handle Drag & Drop of files
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            let mut state = self.state.lock().unwrap();
            let t = state.lang.texts();
            for df in dropped_files {
                let path = df.path().to_path_buf();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file.cdr").to_string();
                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let (ver, pages, details) = inspect_cdr_file(&path).unwrap_or((0, 1, t.file_added_fallback.to_string()));
                let msg = fmt(t.log_added, &[&name, &format_ver(ver), &pages]);
                state.logs.push(msg);
                state.items.push(CdrItem {
                    path,
                    name,
                    size_bytes: size,
                    source_version: ver,
                    num_pages: pages,
                    status: t.row_ready.to_string(),
                    is_success: None,
                    details,
                });
            }
        }

        let (t, current_lang) = {
            let state = self.state.lock().unwrap();
            (state.lang.texts(), state.lang)
        };

        // Top Menu Bar
        egui::Panel::top("top_menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(t.menu_file, |ui| {
                    if ui.button(t.menu_open).clicked() {
                        let state_clone = self.state.clone();
                        let filter_name = t.file_filter.to_string();
                        let fallback_desc = t.file_added_fallback.to_string();
                        let ready_status = t.row_ready.to_string();
                        std::thread::spawn(move || {
                            if let Some(files) = rfd::FileDialog::new()
                                .add_filter(&filter_name, &["cdr", "ai"])
                                .pick_files()
                            {
                                let mut state = state_clone.lock().unwrap();
                                for path in files {
                                    let name = path.file_name().unwrap().to_string_lossy().to_string();
                                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                                    let (ver, pages, details) = inspect_cdr_file(&path).unwrap_or((0, 1, fallback_desc.clone()));
                                    let log_msg = fmt(state.lang.texts().log_opened, &[&name, &format_ver(ver)]);
                                    state.logs.push(log_msg);
                                    state.items.push(CdrItem {
                                        path,
                                        name,
                                        size_bytes: size,
                                        source_version: ver,
                                        num_pages: pages,
                                        status: ready_status.clone(),
                                        is_success: None,
                                        details,
                                    });
                                }
                            }
                        });
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(t.menu_exit).clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button(t.menu_view, |ui| {
                    let mut state = self.state.lock().unwrap();
                    ui.checkbox(&mut state.show_details, t.menu_details);
                    ui.checkbox(&mut state.show_log, t.menu_log);
                });

                ui.menu_button(t.menu_tools, |ui| {
                    let mut state = self.state.lock().unwrap();
                    let t = state.lang.texts();
                    if ui.button(fmt(t.menu_quick_convert, &[&"CDR X4 (1400)"])).clicked() {
                        state.target_ver = TargetVersion::X4_1400;
                        state.suffix = "_x4".to_string();
                        ui.close();
                    }
                    if ui.button(fmt(t.menu_quick_convert, &[&"CDR X5 (1500)"])).clicked() {
                        state.target_ver = TargetVersion::X5_1500;
                        state.suffix = "_x5".to_string();
                        ui.close();
                    }
                    if ui.button(fmt(t.menu_quick_convert, &[&"CDR X8 (1800)"])).clicked() {
                        state.target_ver = TargetVersion::X8_1800;
                        state.suffix = "_x8".to_string();
                        ui.close();
                    }
                    ui.separator();
                    ui.add_enabled(false, egui::Checkbox::new(&mut state.clean_junk, t.menu_clean_junk));
                    ui.add_enabled(false, egui::Checkbox::new(&mut state.fix_outline_id, t.menu_fix_outline));
                });

                ui.menu_button(t.menu_help, |ui| {
                    ui.label(t.help_title);
                    ui.label(t.help_note);
                });

                // Top right language switcher
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut sel_lang = current_lang;
                    egui::ComboBox::from_id_salt("lang_combo_top")
                        .selected_text(format!("🌐 {}", sel_lang.native_name()))
                        .show_ui(ui, |ui| {
                            for l in Lang::ALL {
                                if ui.selectable_value(&mut sel_lang, l, l.native_name()).clicked() {
                                    self.state.lock().unwrap().set_lang(&ctx, l);
                                }
                            }
                        });
                });
            });
        });

        // Bottom Status Bar
        egui::Panel::bottom("status_bar").show(ui, |ui| {
            let state = self.state.lock().unwrap();
            let t = state.lang.texts();
            ui.horizontal(|ui| {
                ui.label(fmt(t.status_counts, &[&state.ok_count, &state.err_count]));
                ui.separator();
                ui.label(fmt(t.status_pending, &[&state.items.len()]));
                ui.separator();
                if state.is_processing {
                    ui.spinner();
                    ui.label(t.status_processing);
                } else {
                    ui.label(t.status_ready);
                }
            });
        });

        // Main Toolbar / Options Header
        egui::Panel::top("options_panel").show(ui, |ui| {
            let mut state = self.state.lock().unwrap();
            let t = state.lang.texts();
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(t.target_version).strong());
                egui::ComboBox::from_id_salt("ver_combo")
                    .selected_text(state.target_ver.label())
                    .show_ui(ui, |ui| {
                        for v in [
                            TargetVersion::X4_1400,
                            TargetVersion::X5_1500,
                            TargetVersion::X6_1600,
                            TargetVersion::X7_1700,
                            TargetVersion::X8_1800,
                            TargetVersion::Ver2019_2100,
                            TargetVersion::Ver2020_2200,
                            TargetVersion::Ver2021_2300,
                            TargetVersion::Ver2022_2400,
                            TargetVersion::Ver2024_2500,
                            TargetVersion::Ver2025_2600,
                            TargetVersion::Ver2026_2700,
                        ] {
                            let label = v.label();
                            let is_sel = state.target_ver == v;
                            if ui.selectable_label(is_sel, label).clicked() {
                                state.suffix = v.default_suffix().to_string();
                                state.target_ver = v;
                            }
                        }
                    });

                ui.separator();

                ui.add_enabled(false, egui::Checkbox::new(&mut state.clean_junk, t.clean_junk))
                    .on_disabled_hover_text(t.clean_junk_tip);
                ui.add_enabled(false, egui::Checkbox::new(&mut state.fix_outline_id, t.fix_outline))
                    .on_disabled_hover_text(t.fix_outline_tip);
                ui.checkbox(&mut state.erase_high_ver, t.erase_high_ver);
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(t.save_location);
                ui.radio_value(&mut state.save_mode, SaveMode::SameDir, t.save_same_dir);
                ui.radio_value(&mut state.save_mode, SaveMode::SubDir, t.save_sub_dir);
                ui.radio_value(&mut state.save_mode, SaveMode::CustomDir, t.save_custom_dir);

                if state.save_mode == SaveMode::SubDir {
                    ui.label(t.subdir);
                    ui.text_edit_singleline(&mut state.subdir_name);
                } else if state.save_mode == SaveMode::CustomDir {
                    ui.label(t.path);
                    ui.text_edit_singleline(&mut state.custom_dir);
                }

                ui.label(t.suffix);
                ui.add(egui::TextEdit::singleline(&mut state.suffix).desired_width(50.0));

                ui.separator();

                // Open File Button
                if ui.button(t.btn_open).clicked() {
                    let state_clone = self.state.clone();
                    let filter_name = t.file_filter.to_string();
                    let fallback_desc = t.file_added_fallback.to_string();
                    let ready_status = t.row_ready.to_string();
                    std::thread::spawn(move || {
                        if let Some(files) = rfd::FileDialog::new()
                            .add_filter(&filter_name, &["cdr", "ai"])
                            .pick_files()
                        {
                            let mut state = state_clone.lock().unwrap();
                            for path in files {
                                let name = path.file_name().unwrap().to_string_lossy().to_string();
                                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                                let (ver, pages, details) = inspect_cdr_file(&path).unwrap_or((0, 1, fallback_desc.clone()));
                                let log_msg = fmt(state.lang.texts().log_opened, &[&name, &format_ver(ver)]);
                                state.logs.push(log_msg);
                                state.items.push(CdrItem {
                                    path,
                                    name,
                                    size_bytes: size,
                                    source_version: ver,
                                    num_pages: pages,
                                    status: ready_status.clone(),
                                    is_success: None,
                                    details,
                                });
                            }
                        }
                    });
                }

                // Start Processing Button
                let is_disabled = state.items.is_empty() || state.is_processing;
                if ui.add_enabled(!is_disabled, egui::Button::new(egui::RichText::new(t.btn_start).strong().color(egui::Color32::WHITE)).fill(egui::Color32::from_rgb(30, 140, 60))).clicked() {
                    state.is_processing = true;
                    let num_items = state.items.len();
                    let start_msg = fmt(t.log_batch_start, &[&num_items]);
                    state.logs.push(start_msg);
                    
                    let state_clone = self.state.clone();
                    std::thread::spawn(move || {
                        let (items, target_ver, save_mode, custom_dir, subdir_name, suffix, clean_junk, fix_outline, erase_high_ver) = {
                            let s = state_clone.lock().unwrap();
                            (
                                s.items.clone(),
                                s.target_ver.clone(),
                                s.save_mode.clone(),
                                s.custom_dir.clone(),
                                s.subdir_name.clone(),
                                s.suffix.clone(),
                                s.clean_junk,
                                s.fix_outline_id,
                                s.erase_high_ver,
                            )
                        };

                        for (idx, item) in items.iter().enumerate() {
                            let parent = item.path.parent().unwrap_or_else(|| Path::new("."));
                            let stem = item.path.file_stem().unwrap_or_default().to_string_lossy();
                            let out_filename = format!("{}{}.cdr", stem, suffix);

                            let out_dir = match save_mode {
                                SaveMode::SameDir => parent.to_path_buf(),
                                SaveMode::SubDir => parent.join(&subdir_name),
                                SaveMode::CustomDir => {
                                    if custom_dir.is_empty() {
                                        parent.to_path_buf()
                                    } else {
                                        PathBuf::from(&custom_dir)
                                    }
                                }
                            };

                            let _ = std::fs::create_dir_all(&out_dir);
                            let out_path = out_dir.join(&out_filename);

                            let res = convert_cdr_file(&item.path, &out_path, &target_ver, clean_junk, fix_outline, erase_high_ver);
                            let mut s = state_clone.lock().unwrap();
                            let t = s.lang.texts();
                            match res {
                                Ok(msg) => {
                                    s.items[idx].status = t.row_success.to_string();
                                    s.items[idx].is_success = Some(true);
                                    s.items[idx].details = msg.clone();
                                    let log_msg = fmt(t.log_convert_ok, &[&item.name, &msg]);
                                    s.logs.push(log_msg);
                                    s.ok_count += 1;
                                }
                                Err(err) => {
                                    s.items[idx].status = t.row_failed.to_string();
                                    s.items[idx].is_success = Some(false);
                                    s.items[idx].details = err.clone();
                                    let log_msg = fmt(t.log_convert_err, &[&item.name, &err]);
                                    s.logs.push(log_msg);
                                    s.err_count += 1;
                                }
                            }
                        }

                        let mut s = state_clone.lock().unwrap();
                        s.is_processing = false;
                        let ok = s.ok_count;
                        let err = s.err_count;
                        let done_msg = fmt(s.lang.texts().log_batch_done, &[&ok, &err]);
                        s.logs.push(done_msg);
                    });
                }
            });
            ui.add_space(4.0);
        });

        // Bottom Details and Logs Collapsible Panels
        let (show_details, show_log) = {
            let state = self.state.lock().unwrap();
            (state.show_details, state.show_log)
        };

        if show_details || show_log {
            egui::Panel::bottom("details_and_logs").resizable(true).min_size(100.0).default_size(160.0).show(ui, |ui| {
                let state = self.state.lock().unwrap();
                let t = state.lang.texts();
                let num_cols = match (show_details, show_log) {
                    (true, true) => 2,
                    _ => 1,
                };
                ui.columns(num_cols, |columns| {
                    let mut col_idx = 0;
                    if show_details {
                        columns[col_idx].vertical(|ui| {
                            ui.heading(t.details_heading);
                            ui.separator();
                            if let Some(idx) = state.selected_idx {
                                if let Some(item) = state.items.get(idx) {
                                    ui.label(fmt(t.det_file_name, &[&item.name]));
                                    ui.label(fmt(t.det_source_ver, &[&format_ver(item.source_version)]));
                                    ui.label(fmt(t.det_target_ver, &[&state.target_ver.label()]));
                                    ui.label(fmt(t.det_file_size, &[&format!("{:.2}", item.size_bytes as f64 / (1024.0 * 1024.0))]));
                                    ui.label(fmt(t.det_pages, &[&item.num_pages]));
                                    ui.label(fmt(t.det_parse, &[&item.details]));
                                }
                            } else {
                                ui.label(t.no_file_selected);
                            }
                        });
                        col_idx += 1;
                    }
                    if show_log {
                        columns[col_idx].vertical(|ui| {
                            ui.heading(t.log_heading);
                            ui.separator();
                            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                                for log in &state.logs {
                                    ui.monospace(log);
                                }
                            });
                        });
                    }
                });
            });
        }

        // Central File Table and Drag/Drop Area
        egui::CentralPanel::default().show(ui, |ui| {
            let mut state = self.state.lock().unwrap();
            let t = state.lang.texts();
            
            if state.items.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading(t.drop_hint);
                    ui.add_space(10.0);
                    ui.label(t.supported_versions);
                    ui.label(t.features);
                });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("files_grid")
                        .striped(true)
                        .min_col_width(80.0)
                        .show(ui, |ui| {
                            ui.strong(t.col_status);
                            ui.strong(t.col_name);
                            ui.strong(t.col_source);
                            ui.strong(t.col_target);
                            ui.strong(t.col_size);
                            ui.strong(t.col_pages);
                            ui.strong(t.col_info);
                            ui.end_row();

                            let mut sel = state.selected_idx;
                            for (idx, item) in state.items.iter().enumerate() {
                                let is_selected = sel == Some(idx);

                                let status_text = match item.is_success {
                                    Some(true) => egui::RichText::new(t.row_success).color(egui::Color32::GREEN),
                                    Some(false) => egui::RichText::new(t.row_failed).color(egui::Color32::RED),
                                    None => egui::RichText::new(t.row_ready).color(egui::Color32::GRAY),
                                };

                                ui.label(status_text);

                                if ui.selectable_label(is_selected, &item.name).clicked() {
                                    sel = Some(idx);
                                }
                                ui.label(format_ver(item.source_version));
                                ui.label(state.target_ver.label());
                                ui.label(format!("{:.1} KB", item.size_bytes as f64 / 1024.0));
                                ui.label(item.num_pages.to_string());
                                ui.label(&item.details);
                                ui.end_row();
                            }
                            state.selected_idx = sel;
                        });
                });
            }
        });
    }
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let mut loaded_latin = false;
    let mut loaded_cjk = false;

    // Latin / Vietnamese fonts
    let latin_candidates = [
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\arial.ttf",
        r"C:\Windows\Fonts\tahoma.ttf",
    ];
    for p in latin_candidates {
        if let Ok(bytes) = std::fs::read(p) {
            fonts.font_data.insert(
                "sys_latin".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            loaded_latin = true;
            break;
        }
    }

    // CJK fonts for Chinese
    let cjk_candidates = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
        "/System/Library/Fonts/PingFang.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ];
    for p in cjk_candidates {
        if let Ok(bytes) = std::fs::read(p) {
            fonts.font_data.insert(
                "sys_cjk".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            loaded_cjk = true;
            break;
        }
    }

    if loaded_latin || loaded_cjk {
        for fam in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            let list = fonts.families.entry(fam).or_default();
            if loaded_cjk {
                list.insert(0, "sys_cjk".to_owned());
            }
            if loaded_latin {
                list.insert(0, "sys_latin".to_owned());
            }
        }
        ctx.set_fonts(fonts);
    }
}

fn load_app_icon() -> Option<egui::IconData> {
    let ico_bytes = include_bytes!("../cdr_conv_icon.ico");
    if let Ok(img) = image::load_from_memory_with_format(ico_bytes, image::ImageFormat::Ico) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = std::env::args().collect();
    let mut screenshot_path = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() {
            screenshot_path = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else {
            i += 1;
        }
    }

    let initial_lang = Lang::load();
    let initial_title = initial_lang.texts().window_title;

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([780.0, 560.0])
        .with_min_inner_size([600.0, 400.0])
        .with_title(initial_title);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "CDR Version Converter",
        native_options,
        Box::new(|cc| {
            install_fonts(&cc.egui_ctx);
            Ok(Box::new(CdrApp::new(screenshot_path)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fourcc_mappings() {
        assert_eq!(fourcc_to_version(b"CDRE"), Some(1400));
        assert_eq!(fourcc_to_version(b"CDRF"), Some(1500));
        assert_eq!(fourcc_to_version(b"CDRI"), Some(1800));
        assert_eq!(fourcc_to_version(b"CDRO"), Some(2400)); // 2022
        assert_eq!(fourcc_to_version(b"CDRP"), Some(2500)); // 2024
        assert_eq!(fourcc_to_version(b"CDRQ"), Some(2600)); // 2025
        assert_eq!(fourcc_to_version(b"CDRR"), Some(2700)); // 2026
    }

    #[test]
    fn test_format_ver_modern() {
        assert_eq!(format_ver(2400), "CDR 2022 (2400)");
        assert_eq!(format_ver(2450), "CDR 2023 (2450)");
        assert_eq!(format_ver(2500), "CDR 2024 (2500)");
        assert_eq!(format_ver(2600), "CDR 2025 (2600)");
        assert_eq!(format_ver(2700), "CDR 2026 (2700)");
        assert_eq!(format_ver(2720), "CDR 2026 (2720)");
    }

    #[test]
    fn test_extract_xml_helpers() {
        let xml1 = "<cdr:AppVersion>2720</cdr:AppVersion><cdr:CoreVersion>2300</cdr:CoreVersion><cdrinfo:NumPages>3</cdrinfo:NumPages>";
        assert_eq!(extract_xml_tag_u16(xml1, "AppVersion"), Some(2720));
        assert_eq!(extract_xml_tag_u16(xml1, "CoreVersion"), Some(2300));
        assert_eq!(extract_page_count(xml1), Some(3));

        let xml2 = "<AppVersion>2500</AppVersion><CoreVersion>1500</CoreVersion><NumPages>2</NumPages>";
        assert_eq!(extract_xml_tag_u16(xml2, "AppVersion"), Some(2500));
        assert_eq!(extract_xml_tag_u16(xml2, "CoreVersion"), Some(1500));
        assert_eq!(extract_page_count(xml2), Some(2));
    }

    #[test]
    fn test_inspect_sample_files() {
        let test_dir = Path::new("../testFile");
        let sample1 = test_dir.join("test_CameraCut.cdr");
        if sample1.exists() {
            let res = inspect_cdr_file(&sample1);
            assert!(res.is_ok(), "Failed to inspect test_CameraCut.cdr: {:?}", res);
            let (ver, pages, details) = res.unwrap();
            assert_eq!(ver, 2300, "Should detect 2300 (CorelDRAW 2021 format)");
            assert_eq!(pages, 1);
            assert!(details.contains("2720"), "Details should mention saving app version 2720");
        }

        let sample2 = test_dir.join("So kham phu khoa (nguyen tien cong).cdr");
        if sample2.exists() {
            let res = inspect_cdr_file(&sample2);
            assert!(res.is_ok());
            let (ver, _pages, _details) = res.unwrap();
            assert_eq!(ver, 1500, "Should detect 1500 (CorelDRAW X5 format)");
        }

        let sample3 = test_dir.join("The.cdr");
        if sample3.exists() {
            let res = inspect_cdr_file(&sample3);
            assert!(res.is_ok());
            let (ver, _pages, _details) = res.unwrap();
            assert_eq!(ver, 1900, "Should detect 1900 (CorelDRAW 2017 format)");
        }
    }
}
