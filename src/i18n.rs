//! Simple i18n support: Vietnamese (default), English, Chinese.

use std::fmt::Display;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Vi,
    En,
    Zh,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::Vi, Lang::En, Lang::Zh];

    /// Language name displayed in its own language.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Vi => "Tiếng Việt [Mặc định]",
            Lang::En => "English",
            Lang::Zh => "中文",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Vi => "vi",
            Lang::En => "en",
            Lang::Zh => "zh",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        match code.trim().to_ascii_lowercase().as_str() {
            "vi" => Some(Lang::Vi),
            "en" => Some(Lang::En),
            "zh" => Some(Lang::Zh),
            _ => None,
        }
    }

    pub fn texts(self) -> &'static Texts {
        match self {
            Lang::Vi => &VI,
            Lang::En => &EN,
            Lang::Zh => &ZH,
        }
    }

    fn config_path() -> Option<PathBuf> {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("CdrVersionConverter").join("lang.txt"))
    }

    /// Loads the saved language, falling back to Vietnamese.
    pub fn load() -> Lang {
        Self::config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| Lang::from_code(&s))
            .unwrap_or_default()
    }

    /// Persists the chosen language (best effort).
    pub fn save(self) {
        if let Some(p) = Self::config_path() {
            if let Some(dir) = p.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(p, self.code());
        }
    }
}

/// Replaces each `{}` in `template` with the next argument, in order.
pub fn fmt(template: &str, args: &[&dyn Display]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    let mut it = args.iter();
    while let Some(pos) = rest.find("{}") {
        out.push_str(&rest[..pos]);
        match it.next() {
            Some(a) => out.push_str(&a.to_string()),
            None => out.push_str("{}"),
        }
        rest = &rest[pos + 2..];
    }
    out.push_str(rest);
    out
}

pub struct Texts {
    pub window_title: &'static str,
    // Menu
    pub menu_file: &'static str,
    pub menu_open: &'static str,
    pub menu_exit: &'static str,
    pub menu_view: &'static str,
    pub menu_details: &'static str,
    pub menu_log: &'static str,
    pub menu_tools: &'static str,
    pub menu_quick_convert: &'static str,
    pub menu_clean_junk: &'static str,
    pub menu_fix_outline: &'static str,
    pub menu_settings: &'static str,
    pub language: &'static str,
    pub menu_help: &'static str,
    pub help_title: &'static str,
    pub help_note: &'static str,
    // Status bar
    pub status_counts: &'static str,
    pub status_pending: &'static str,
    pub status_processing: &'static str,
    pub status_ready: &'static str,
    // Options panel
    pub target_version: &'static str,
    pub clean_junk: &'static str,
    pub clean_junk_tip: &'static str,
    pub fix_outline: &'static str,
    pub fix_outline_tip: &'static str,
    pub erase_high_ver: &'static str,
    pub save_location: &'static str,
    pub save_same_dir: &'static str,
    pub save_sub_dir: &'static str,
    pub save_custom_dir: &'static str,
    pub subdir: &'static str,
    pub path: &'static str,
    pub suffix: &'static str,
    pub btn_open: &'static str,
    pub btn_start: &'static str,
    pub file_filter: &'static str,
    // Details / log panel
    pub details_heading: &'static str,
    pub det_file_name: &'static str,
    pub det_source_ver: &'static str,
    pub det_target_ver: &'static str,
    pub det_file_size: &'static str,
    pub det_pages: &'static str,
    pub det_parse: &'static str,
    pub no_file_selected: &'static str,
    pub log_heading: &'static str,
    // Central panel
    pub drop_hint: &'static str,
    pub supported_versions: &'static str,
    pub features: &'static str,
    pub col_status: &'static str,
    pub col_name: &'static str,
    pub col_source: &'static str,
    pub col_target: &'static str,
    pub col_size: &'static str,
    pub col_pages: &'static str,
    pub col_info: &'static str,
    pub row_success: &'static str,
    pub row_failed: &'static str,
    pub row_ready: &'static str,
    // Logs
    pub log_init: &'static str,
    pub log_ready: &'static str,
    pub log_added: &'static str,
    pub log_opened: &'static str,
    pub log_batch_start: &'static str,
    pub log_convert_ok: &'static str,
    pub log_convert_err: &'static str,
    pub log_batch_done: &'static str,
    pub log_lang_changed: &'static str,
    pub file_added_fallback: &'static str,
}

pub static VI: Texts = Texts {
    window_title: "Trình chuyển đổi phiên bản CDR - CDR Version Converter",
    menu_file: "Tệp",
    menu_open: "Mở...   Ctrl-O",
    menu_exit: "Thoát",
    menu_view: "Xem",
    menu_details: "Thông tin chi tiết",
    menu_log: "Nhật ký",
    menu_tools: "Công cụ",
    menu_quick_convert: "Chuyển nhanh sang {}",
    menu_clean_junk: "Dọn rác tệp",
    menu_fix_outline: "Sửa lỗi ID đường viền",
    menu_settings: "Cài đặt",
    language: "Ngôn ngữ",
    menu_help: "Trợ giúp",
    help_title: "Trình chuyển đổi phiên bản CDR",
    help_note: "Chỉ thay đổi số phiên bản; dữ liệu bản vẽ không bị ghi lại.",
    status_counts: "Thành công: {}  Lỗi: {}",
    status_pending: "Tệp chờ xử lý: {}",
    status_processing: "Đang xử lý...",
    status_ready: "Sẵn sàng",
    target_version: "Phiên bản đích:",
    clean_junk: "Dọn rác tệp CDR",
    clean_junk_tip: "Chưa khả dụng: chưa rõ quy tắc dọn rác của bản gốc",
    fix_outline: "Sửa lỗi ID đường viền",
    fix_outline_tip: "Chưa khả dụng: chưa rõ quy tắc sửa đường viền của bản gốc",
    erase_high_ver: "Xóa thông tin phần mềm bản cao",
    save_location: "Vị trí lưu:",
    save_same_dir: "1. Cùng thư mục gốc",
    save_sub_dir: "2. Thư mục con",
    save_custom_dir: "3. Thư mục tuyệt đối",
    subdir: "Thư mục con:",
    path: "Đường dẫn:",
    suffix: "Hậu tố:",
    btn_open: "📂 Mở tệp CDR",
    btn_start: "⚡ Bắt đầu xử lý",
    file_filter: "Tệp được hỗ trợ (*.cdr;*.ai)",
    details_heading: "Thông tin chi tiết",
    det_file_name: "Tên tệp: {}",
    det_source_ver: "Phiên bản nguồn: {}",
    det_target_ver: "Phiên bản đích: {}",
    det_file_size: "Kích thước: {} MB",
    det_pages: "Số trang: {}",
    det_parse: "Chi tiết phân tích: {}",
    no_file_selected: "Chưa chọn tệp nào",
    log_heading: "Nhật ký",
    drop_hint: "Kéo thả tệp CDR vào cửa sổ...",
    supported_versions: "Hỗ trợ: CorelDRAW 11 đến 2026 (*.cdr; *.ai)",
    features: "Tính năng: chuyển nhanh sang X4/X8, dọn rác tệp CDR, sửa lỗi ID đường viền, xóa thông tin bản cao",
    col_status: "Trạng thái",
    col_name: "Tên tệp",
    col_source: "Bản nguồn",
    col_target: "Bản đích",
    col_size: "Kích thước",
    col_pages: "Trang",
    col_info: "Thông tin",
    row_success: "✓ Thành công",
    row_failed: "✕ Thất bại",
    row_ready: "Sẵn sàng",
    log_init: "CDR Version Converter v3.5.1 đã khởi động.",
    log_ready: "Sẵn sàng. Kéo thả tệp .cdr vào cửa sổ hoặc bấm 'Mở tệp CDR'.",
    log_added: "Đã thêm tệp: {} ({}, {} trang)",
    log_opened: "Đã mở tệp: {} ({})",
    log_batch_start: "Bắt đầu xử lý hàng loạt {} tệp...",
    log_convert_ok: "✓ Chuyển đổi thành công {}: {}",
    log_convert_err: "✕ Chuyển đổi thất bại {}: {}",
    log_batch_done: "Hoàn tất! Thành công: {}, Lỗi: {}",
    log_lang_changed: "Đã chuyển ngôn ngữ sang: {}",
    file_added_fallback: "Đã thêm tệp",
};

pub static EN: Texts = Texts {
    window_title: "CDR Version Converter",
    menu_file: "File",
    menu_open: "Open...   Ctrl-O",
    menu_exit: "Exit",
    menu_view: "View",
    menu_details: "Details",
    menu_log: "Log",
    menu_tools: "Tools",
    menu_quick_convert: "Quick convert to {}",
    menu_clean_junk: "Clean file junk",
    menu_fix_outline: "Fix outline ID errors",
    menu_settings: "Settings",
    language: "Language",
    menu_help: "Help",
    help_title: "CDR Version Converter",
    help_note: "Version numbers only; the drawing data is not rewritten.",
    status_counts: "OK: {}  Failed: {}",
    status_pending: "Pending files: {}",
    status_processing: "Processing...",
    status_ready: "Ready",
    target_version: "Target version:",
    clean_junk: "Clean CDR file junk",
    clean_junk_tip: "Not available yet: the original's cleanup rules are unknown",
    fix_outline: "Fix outline ID errors",
    fix_outline_tip: "Not available yet: the original's outline-fix rules are unknown",
    erase_high_ver: "Erase newer-version software info",
    save_location: "Save to:",
    save_same_dir: "1. Same folder as source",
    save_sub_dir: "2. Subfolder",
    save_custom_dir: "3. Absolute folder",
    subdir: "Subfolder:",
    path: "Path:",
    suffix: "Suffix:",
    btn_open: "📂 Open CDR files",
    btn_start: "⚡ Start",
    file_filter: "Supported files (*.cdr;*.ai)",
    details_heading: "Details",
    det_file_name: "File name: {}",
    det_source_ver: "Source version: {}",
    det_target_ver: "Target version: {}",
    det_file_size: "File size: {} MB",
    det_pages: "Pages: {}",
    det_parse: "Parse details: {}",
    no_file_selected: "No file selected",
    log_heading: "Log",
    drop_hint: "Drag CDR files into the window...",
    supported_versions: "Supported: CorelDRAW 11 to 2026 (*.cdr; *.ai)",
    features: "Features: one-click X4/X8, clean CDR junk, fix outline IDs, erase newer-version info",
    col_status: "Status",
    col_name: "File name",
    col_source: "Source",
    col_target: "Target",
    col_size: "Size",
    col_pages: "Pages",
    col_info: "Info",
    row_success: "✓ Success",
    row_failed: "✕ Failed",
    row_ready: "Ready",
    log_init: "CDR Version Converter v3.5.1 initialized.",
    log_ready: "Ready for files. Drag .cdr files into the window or click 'Open CDR files'.",
    log_added: "Added file: {} ({}, {} pages)",
    log_opened: "Opened file: {} ({})",
    log_batch_start: "Started batch processing of {} files...",
    log_convert_ok: "✓ Successfully converted {}: {}",
    log_convert_err: "✕ Failed to convert {}: {}",
    log_batch_done: "Batch finished! Success: {}, Errors: {}",
    log_lang_changed: "Language switched to: {}",
    file_added_fallback: "File added",
};

pub static ZH: Texts = Texts {
    window_title: "CDR版本转换器",
    menu_file: "文件(&F)",
    menu_open: "打开(&O)   Ctrl-O",
    menu_exit: "退出(&X)",
    menu_view: "视图(&V)",
    menu_details: "详细信息(&D)",
    menu_log: "日志信息(&L)",
    menu_tools: "工具(&T)",
    menu_quick_convert: "一键转为{}",
    menu_clean_junk: "清理文件垃圾",
    menu_fix_outline: "修复轮廓ID错误",
    menu_settings: "设置(&S)",
    language: "语言",
    menu_help: "帮助(&H)",
    help_title: "CDR 版本转换",
    help_note: "仅修改版本号；不会重写绘图数据。",
    status_counts: "成:{}  败:{}",
    status_pending: "待处理文件: {}",
    status_processing: "处理中...",
    status_ready: "就绪",
    target_version: "转换版本:",
    clean_junk: "清理cdr文件垃圾",
    clean_junk_tip: "暂不可用：原程序的清理规则未知",
    fix_outline: "修复轮廓id错误",
    fix_outline_tip: "暂不可用：原程序的轮廓修复规则未知",
    erase_high_ver: "擦除高版本软件信息",
    save_location: "保存位置:",
    save_same_dir: "1.源文件同目录",
    save_sub_dir: "2.指定子目录",
    save_custom_dir: "3.绝对目录",
    subdir: "子目录:",
    path: "路径:",
    suffix: "后缀:",
    btn_open: "📂 打开CDR文件",
    btn_start: "⚡ 开始处理",
    file_filter: "支持的文件 (*.cdr;*.ai)",
    details_heading: "详细信息",
    det_file_name: "文件名: {}",
    det_source_ver: "源文件版本: {}",
    det_target_ver: "目标版本: {}",
    det_file_size: "文件大小: {} MB",
    det_pages: "页码数量: {}",
    det_parse: "解析详情: {}",
    no_file_selected: "未选中文件",
    log_heading: "日志信息",
    drop_hint: "请拖动cdr文件到窗口...",
    supported_versions: "支持版本: CorelDRAW 11 到 2026 (*.cdr; *.ai)",
    features: "功能特点: 一键转X4/X8、清理cdr文件垃圾、修复轮廓id错误、擦除高版本信息",
    col_status: "状态",
    col_name: "文件名",
    col_source: "源版本",
    col_target: "目标版本",
    col_size: "大小",
    col_pages: "页码",
    col_info: "信息",
    row_success: "✓ 成功",
    row_failed: "✕ 失败",
    row_ready: "就绪",
    log_init: "CDR 版本转换器 v3.5.1 已初始化。",
    log_ready: "就绪。请将 .cdr 文件拖入窗口或点击“打开CDR文件”。",
    log_added: "已添加文件: {} ({}, {} 页)",
    log_opened: "已打开文件: {} ({})",
    log_batch_start: "开始批量处理 {} 个文件...",
    log_convert_ok: "✓ 转换成功 {}: {}",
    log_convert_err: "✕ 转换失败 {}: {}",
    log_batch_done: "批量处理完成！成功: {}，失败: {}",
    log_lang_changed: "语言已切换为: {}",
    file_added_fallback: "已添加文件",
};
