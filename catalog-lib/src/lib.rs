pub mod error;
use std::{
    borrow::Cow,
    env::{self},
    fs::{copy, File, OpenOptions},
    io::{self, BufReader, BufWriter},
    os::windows::process::CommandExt,
    path::PathBuf,
    process::Command,
    ptr::null_mut,
};

use data_encoding::BASE64;
use error::CatalogError;
use sha3::{Digest, Sha3_384};
use walkdir::WalkDir;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::RECT,
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
                COINIT_APARTMENTTHREADED,
            },
            Registry::{self, HKEY, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, REG_SZ, REG_VALUE_TYPE},
            Threading::CREATE_NO_WINDOW,
        },
        UI::{
            Shell::{FileOpenDialog, IFileOpenDialog, SIGDN_FILESYSPATH},
            WindowsAndMessaging::{GetDesktopWindow, GetForegroundWindow, GetWindowRect},
        },
    },
};
use xml::{attribute::Attribute, reader::EventReader, writer::XmlEvent, EventWriter};

/// 获取桌面窗口的大小
pub fn get_desktop_window_size() -> (i32, i32) {
    // 获取桌面窗口句柄
    let desktop_window = unsafe { GetDesktopWindow() };

    // 获取桌面窗口的矩形区域
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    unsafe {
        let _ = GetWindowRect(desktop_window, &mut rect);
    }

    // 计算桌面窗口的宽度和高度
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    (width, height)
}

pub fn open_file_dialog() -> windows::core::Result<String> {
    unsafe {
        let _ = CoInitializeEx(Some(null_mut()), COINIT_APARTMENTTHREADED);
        let file_dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
        let hwnd = GetForegroundWindow();
        file_dialog.Show(hwnd)?;
        let result = file_dialog.GetResult()?;
        let file_path = result.GetDisplayName(SIGDN_FILESYSPATH)?.to_string()?;
        CoUninitialize();
        Ok(file_path)
    }
}

pub struct CatalogInfo {
    pub cab_path: String,
    pub ic_path: String,
}

impl CatalogInfo {
    fn new(cab_path: String, ic_path: String) -> Self {
        CatalogInfo { cab_path, ic_path }
    }
}

pub fn get_catalog_and_ic_paths() -> Result<CatalogInfo, CatalogError> {
    let current_dir = env::current_dir().unwrap_or_else(|e| {
        eprintln!("Failed to get current directory: {}", e);
        PathBuf::from(".")
    });
    let mut cab_files = Vec::new();
    let mut exe_files = Vec::new();
    for entry in WalkDir::new(current_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let f_name = String::from(entry.file_name().to_string_lossy());
        if f_name.ends_with(".cab") {
            cab_files.push(f_name);
        } else if f_name.ends_with(".exe") && f_name.to_lowercase().contains("invc") {
            exe_files.push(f_name);
        }
    }
    match (cab_files.as_slice(), exe_files.as_slice()) {
        ([cab_file], [exe_file]) => Ok(CatalogInfo::new(cab_file.clone(), exe_file.clone())),
        ([cab_file], []) => Ok(CatalogInfo::new(
            cab_file.clone(),
            "No invc.exe file found".into(),
        )),
        ([], [exe_file]) => Ok(CatalogInfo::new(
            "No .cab file found".into(),
            exe_file.clone(),
        )),
        ([], []) => Err(CatalogError::NoFilesFound),
        (cab_files, exe_files) if cab_files.len() > 1 || exe_files.len() > 1 => {
            Err(CatalogError::MultipleFilesFound)
        }
        (_, _) => Err(CatalogError::Unexpected),
    }
}

pub fn cab_to_xml(cab_path: &str) -> Result<String, CatalogError> {
    let cmd_str = format!("expand.exe -R {cab_path} > nul ");
    let output = Command::new("cmd")
        .creation_flags(CREATE_NO_WINDOW.0)
        .arg("/c")
        .arg(cmd_str)
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("cmd exec error: {}", e)))?;

    if !output.status.success() {
        return Err(CatalogError::ParseError("cmd command failed".into()));
    }

    let xml_path = format!("{}.xml", cab_path.trim_end_matches(".cab"));
    // 读取原始 XML 文件
    let input_file = File::open(&xml_path)?;
    let input_reader = BufReader::new(input_file);

    // 打开输出文件以写入
    let output_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&xml_path)?;

    let reader = EventReader::new(input_reader);

    let mut event_writer = EventWriter::new(BufWriter::new(output_file));

    let mut in_software = false;
    let mut in_maniface = false;

    for event in reader.into_iter() {
        match event {
            Ok(event) => {
                if let Some(w_event) = event.as_writer_event() {
                    match w_event {
                        XmlEvent::StartElement {
                            name,
                            ref attributes,
                            ref namespace,
                        } => {
                            let mut new_attributes = Vec::new();
                            if name.local_name == "SoftwareComponent" {
                                in_software = true;
                                // 修改 path 属性
                                new_attributes.clear();
                                for attr in attributes.iter() {
                                    if attr.name.local_name == "path" {
                                        let new_value =
                                            attr.value.split("\\").nth(1).unwrap_or_default();
                                        new_attributes.push(Attribute::new(attr.name, new_value));
                                    } else {
                                        new_attributes.push(*attr);
                                    }
                                }
                                match event_writer.write(XmlEvent::StartElement {
                                    name,
                                    attributes: Cow::Owned(new_attributes),
                                    namespace: namespace.clone(),
                                }) {
                                    Ok(_) => {}
                                    Err(e) => eprintln!("Error: {}", e),
                                };
                            } else if name.local_name == "Manufacturer" {
                                in_maniface = true;
                                new_attributes.clear();
                                for attr in attributes.iter() {
                                    if attr.name.local_name == "baseLocation" {
                                        new_attributes.push(Attribute::new(attr.name, ""));
                                    } else {
                                        new_attributes.push(*attr);
                                    }
                                }
                            } else {
                                match event_writer.write(w_event) {
                                    Ok(_) => {}
                                    Err(e) => eprintln!("Error: {}", e),
                                };
                            }
                        }
                        XmlEvent::EndElement { name, .. } => {
                            if let Some(name) = name {
                                if in_software && name.local_name == "path" {
                                    in_software = false;
                                } else if in_maniface && name.local_name == "Manufacturer" {
                                    in_maniface = false;
                                }
                                match event_writer.write(w_event) {
                                    Ok(_) => {}
                                    Err(e) => eprintln!("Error: {}", e),
                                };
                            }
                        }
                        _ => {
                            match event_writer.write(w_event) {
                                Ok(_) => {}
                                Err(e) => eprintln!("Error: {}", e),
                            };
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(xml_path)
    // let sp_cab = cab_path.splitn(2, ".").collect::<Vec<_>>();
    // format!("{}{}", sp_cab[0], ".xml")
}

fn str_to_pcwstr(s: &str) -> PCWSTR {
    let result = s
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    PCWSTR::from_raw(result.as_ptr())
}

pub fn open_reg_subkey(sub_key: &str) -> Result<HKEY, String> {
    let mut new_key: HKEY = HKEY::default();
    let _sub_key = str_to_pcwstr(sub_key);
    let res = unsafe {
        Registry::RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            _sub_key,
            0,
            KEY_ALL_ACCESS,
            &mut new_key,
        )
    }
    .is_ok();
    if res {
        Ok(new_key)
    } else {
        Err(format!("Failed to turn on the \"{}\" key", sub_key))
    }
}

pub trait ToOptionU8Slice {
    fn to_option_u8(&self) -> Option<&[u8]>;
}

impl ToOptionU8Slice for &str {
    fn to_option_u8(&self) -> Option<&[u8]> {
        if self.is_empty() {
            None
        } else {
            Some(self.as_bytes())
        }
    }
}

pub fn set_reg_vaule<T: ToOptionU8Slice>(
    sub_key: HKEY,
    value_name: &str,
    dw_type: REG_VALUE_TYPE,
    vaule: T,
) -> bool {
    let _value_name = str_to_pcwstr(value_name);

    unsafe { Registry::RegSetValueExW(sub_key, _value_name, 0, dw_type, vaule.to_option_u8()) }
        .is_ok()
}

pub fn delete_reg_key_vaule(key: HKEY, sub_key: Option<&str>, value_names: Vec<&str>) {
    match sub_key {
        Some(_sub_key) => unsafe {
            let _ = Registry::RegDeleteKeyExW(key, str_to_pcwstr(_sub_key), 0x0100, 0);
        },
        None => {
            for lpvaluename in value_names {
                unsafe {
                    let _ = Registry::RegDeleteValueW(key, str_to_pcwstr(lpvaluename));
                };
            }
        }
    }
}

pub fn get_hash_sha384(xml_path: String) -> Result<Option<String>, std::io::Error> {
    let file = File::open(&xml_path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha3_384::new();
    let _ = io::copy(&mut reader, &mut hasher);
    let hash = hasher.finalize();
    if let Some(base64_str) = BASE64.encode(&hash[..]).strip_suffix("=") {
        Ok(Some(format!(
            r#"{{"Key":{},"Value":{}\}}"#,
            xml_path, base64_str
        )))
    } else {
        Ok(None)
    }
}

pub fn handle_reg(str_hash: &str, software: &Software) {
    match software {
        Software::DellUpdate { name } => {
            let service_key = open_reg_subkey(r#"SOFTWARE\Dell\UpdateService\Service"#).unwrap();
            set_reg_vaule(service_key, "CustomCatalogHashValues", REG_SZ, str_hash);
            let service_vaule = vec![
                "LastCheckTimestamp",
                "LastUpdateTimestamp",
                "CatalogTimestamp",
                "CatalogTimestamp",
            ];
            delete_reg_key_vaule(service_key, Some("IgnoreList"), service_vaule.clone());
            delete_reg_key_vaule(service_key, None, service_vaule);
            open_software(name);
            todo!()
        }
        Software::DellCommandUpdate { .. } => todo!(),
    }
}

pub fn open_software(name: &String) {
    let _ = Command::new("start")
        .arg(name)
        .spawn()
        .expect("Failed to open software");
}

pub enum Software {
    DellUpdate { name: String },
    DellCommandUpdate { name: String },
}

const DCU_PATH: &str = r#"SOFTWARE\Dell\UpdateService\Clients\CommandUpdate"#;
const DU_PATH: &str = r#"SOFTWARE\Dell\UpdateService\Clients\Update"#;
pub fn du_or_dcu() -> Option<Software> {
    match open_reg_subkey(DCU_PATH) {
        Ok(_) => Some(Software::DellCommandUpdate {
            name: "Dell Command Update".to_string(),
        }),
        Err(_) => match open_reg_subkey(DU_PATH) {
            Ok(_) => Some(Software::DellUpdate {
                name: "Dell Update".to_string(),
            }),
            Err(_) => None,
        },
    }
}

pub async fn process() {
    match get_catalog_and_ic_paths() {
        Ok(catalog_info) => {
            let xml_path = cab_to_xml(&catalog_info.cab_path);
            if let Ok(xml_path) = xml_path {
                let op_str_hash = get_hash_sha384(xml_path).unwrap_or_default();
                let hash = op_str_hash.unwrap_or_default();
                let _ = copy(
                    &catalog_info.ic_path,
                    r"C:\Program Files (x86)\Dell\UpdateService\Service\InvColPC.exe",
                );
                if let Some(ref software) = du_or_dcu() {
                    match software {
                        Software::DellUpdate { name } => {
                            println!("{}", name);
                            handle_reg(&hash, software)
                        }
                        Software::DellCommandUpdate { name } => {
                            println!("{}", name);
                            handle_reg(&hash, software)
                        }
                    }
                }
            } else {
            }
        }
        Err(_) => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let cab_path = get_catalog_and_ic_paths().unwrap().cab_path.clone();
        let xml = cab_to_xml(&cab_path);
    }
}
