pub mod error;
// pub mod test_xml;
use std::{
    borrow::Cow,
    env::{self},
    ffi::OsStr,
    fs::{copy, File},
    io::{self, BufReader, BufWriter},
    iter::once,
    os::windows::ffi::OsStrExt,
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

#[derive(Debug, Default, Clone)]
pub struct CatalogInfo {
    pub cab_path: String,
    pub ic_path: String,
}

impl CatalogInfo {
    fn new(cab_path: String, ic_path: String) -> Self {
        CatalogInfo { cab_path, ic_path }
    }
}

pub fn get_catalog_and_ic_paths(current_dir: PathBuf) -> Result<CatalogInfo, CatalogError> {
    println!("{:?}", current_dir);
    let mut cab_files = Vec::new();
    let mut exe_files = Vec::new();
    for entry in WalkDir::new(current_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let f_name = String::from(entry.path().to_string_lossy());
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

pub fn cab_to_xml(cab_path: &str) -> Result<PathBuf, CatalogError> {
    let cmd_str = format!("expand.exe -R {cab_path}");
    let output = Command::new("cmd")
        // .creation_flags(CREATE_NO_WINDOW.0)
        .arg("/c")
        .arg(cmd_str)
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("cmd exec error: {}", e)))?;

    if !output.status.success() {
        return Err(CatalogError::ParseError("cmd command failed".into()));
    }

    let xml_path = format!("{}.xml", cab_path.trim_end_matches(".cab"));
    Ok(xml_path.into())
}

fn handle_xml(xml_path: PathBuf) -> Result<PathBuf, CatalogError> {
    // println!("handle_xml--{:?}", xml_path);
    let input_file = File::open(&xml_path)?;
    let input_reader = BufReader::new(input_file);
    let reader = EventReader::new(input_reader);
    let mut output_xml_path = xml_path.parent().unwrap().to_path_buf();
    let file_name = xml_path.file_name().unwrap();
    output_xml_path.push(Into::<PathBuf>::into(format!(
        "_{}",
        file_name.to_string_lossy()
    )));
    // println!("output_xml_path--{:?}", output_xml_path);
    let output_file = File::create(&output_xml_path)?;
    let output_writer = BufWriter::new(output_file);
    let mut event_writer = EventWriter::new(BufWriter::new(output_writer));

    let mut in_software = false;
    let mut in_maniface = false;

    let mut depth = 0;

    for event in reader {
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
                            // println!("StartElement-{}", name.local_name);
                            if name.local_name == "Manifest" {
                                // in_maniface = true;
                                new_attributes.clear();
                                for attr in attributes.iter() {
                                    if attr.name.local_name == "baseLocation" {
                                        new_attributes.push(Attribute::new(attr.name, ""));
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
                                    Err(e) => {
                                        eprintln!("Error: StartElement-Manifest--{}", e)
                                    }
                                };
                            } else if name.local_name == "SoftwareComponent" {
                                // in_software = true;
                                // 修改 path 属性
                                new_attributes.clear();
                                for attr in attributes.iter() {
                                    if attr.name.local_name == "path" {
                                        let new_value =
                                            attr.value.split("/").nth(2).unwrap_or_default();
                                        println!("SoftwareComponent-new_value--{}", new_value);
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
                                    Err(e) => {
                                        eprintln!("Error: StartElement-SoftwareComponent--{}", e)
                                    }
                                };
                            } else {
                                match event_writer.write(w_event) {
                                    Ok(_) => {}
                                    Err(e) => eprintln!("Error: StartElement-else--{}", e),
                                };
                            }
                        }
                        XmlEvent::EndElement { name } => {
                            match event_writer.write(w_event.clone()) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Error: EndElement-else-event{:?}-{}", w_event, e)
                                }
                            };
                            if let Some(name) = name {
                                // if in_software && name.local_name == "SoftwareComponent" {
                                //     in_software = false;
                                // } else if in_maniface && name.local_name == "Manifest" {
                                //     in_maniface = false;
                                // }
                                // match event_writer.write(w_event.clone()) {
                                //     Ok(_) => {}
                                //     Err(e) => {
                                //         eprintln!("Error: EndElement-else-event{:?}-{}", w_event, e)
                                //     }
                                // };
                            }
                        }
                        _ => {
                            match event_writer.write(w_event) {
                                Ok(_) => {}
                                Err(e) => panic!("Write error: {e}"),
                            };
                        }
                    }
                }
            }
            Err(e) => eprintln!("match event: {}", e),
        }
    }
    Ok(output_xml_path)
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

pub fn set_reg_vaule(
    sub_key: HKEY,
    value_name: &str,
    dw_type: REG_VALUE_TYPE,
    vaule: &str,
) -> bool {
    let _value_name = str_to_pcwstr(value_name);
    let os_str = OsStr::new(vaule).as_encoded_bytes();
    println!("set_reg_vaule--{:?}", vaule);
    unsafe { Registry::RegSetValueExW(sub_key, _value_name, 0, dw_type, Some(&os_str)) }.is_ok()
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

pub fn get_hash_sha384(xml_path: PathBuf) -> Result<String, std::io::Error> {
    let file = File::open(&xml_path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha3_384::new();
    let _ = io::copy(&mut reader, &mut hasher);
    let hash = hasher.finalize();
    let base64 = BASE64.encode(&hash[..]);
    let base64_str = match base64.strip_suffix("=") {
        Some(temp) => format!(
            r#"{{"CatalogHashValues":[{{"Key":"{}","Value":"{}"}}]}}"#,
            xml_path.to_string_lossy(),
            temp
        ),
        None => format!(
            r#"{{"CatalogHashValues":[{{"Key":"{}","Value":"{}"}}]}}"#,
            xml_path.to_string_lossy(),
            base64
        ),
    };
    println!("get_hash_sha384--{}", base64_str);
    Ok(base64_str)
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
        }
        Software::DellCommandUpdate { name } => {
            let service_key = open_reg_subkey(r#"SOFTWARE\Dell\UpdateService\Service"#).unwrap();
            println!("handle_reg--{}", str_hash);
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
        }
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
            Err(e) => {
                eprintln!("du_or_dcu--{}", e);
                None
            }
        },
    }
}

pub fn get_cur_path() -> PathBuf {
    env::current_dir().unwrap_or_else(|e| {
        eprintln!("Failed to get current directory: {}", e);
        PathBuf::from(".")
    })
}

fn handle() -> Result<(), CatalogError> {
    let catalog_info = get_catalog_and_ic_paths(get_cur_path())?;
    let xml_path = cab_to_xml(&catalog_info.cab_path)?;
    let new_xml_path = handle_xml(xml_path)?;
    let hash = get_hash_sha384(new_xml_path)?;
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
    } else {
        println!("请安装DCU或者DU");
    };
    Ok(())
}

pub async fn process() {
    let _ = handle();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
