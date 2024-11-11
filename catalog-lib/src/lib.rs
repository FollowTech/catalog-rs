pub mod error;
// pub mod test_xml;
use std::{
    borrow::Cow,
    env::{self},
    ffi::OsStr,
    fs::{copy, File},
    io::{self, BufReader, BufWriter},
    path::PathBuf,
    process::Command,
    ptr::null_mut,
};

use data_encoding::BASE64;
use error::CatalogError;
use iced::Size;
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
        },
        UI::{
            Shell::{FileOpenDialog, IFileOpenDialog, SIGDN_FILESYSPATH},
            WindowsAndMessaging::{GetDesktopWindow, GetForegroundWindow, GetWindowRect},
        },
    },
};
use xml::{attribute::Attribute, reader::EventReader, writer::XmlEvent, EventWriter};

#[derive(Debug, Default, Clone)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

/// 获取桌面窗口的大小
pub fn get_window_size() -> Size {
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

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    Size {
        width: (width / 2) as f32,
        height: (height / 2) as f32,
    }
}

fn filename_to_lower_string(file_path: &PathBuf) -> String {
    file_path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_string()
        .to_lowercase()
}

pub fn is_cab_path(file_path: &PathBuf) -> bool {
    filename_to_lower_string(file_path).ends_with("cab")
}

pub fn is_ic_path(file_path: &PathBuf) -> bool {
    let filename = filename_to_lower_string(file_path);
    filename.ends_with("exe") && filename.contains("invc")
}

pub fn open_file_dialog() -> Result<PathBuf, CatalogError> {
    let res = unsafe {
        let _ = CoInitializeEx(Some(null_mut()), COINIT_APARTMENTTHREADED);
        let file_dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
        let hwnd = GetForegroundWindow();
        file_dialog.Show(hwnd)?;
        let result = file_dialog.GetResult()?;
        let file_path = result
            .GetDisplayName(SIGDN_FILESYSPATH)?
            .to_hstring()?
            .to_string_lossy();
        CoUninitialize();
        PathBuf::from(file_path)
    };
    Ok(res)
}

#[derive(Debug, Default, Clone)]
pub struct CatalogInfo {
    pub cab_path: PathBuf,
    pub ic_path: PathBuf,
}

impl From<(PathBuf, PathBuf)> for CatalogInfo {
    fn from(catalog_info: (PathBuf, PathBuf)) -> Self {
        CatalogInfo {
            cab_path: catalog_info.0,
            ic_path: catalog_info.1,
        }
    }
}
pub async fn get_catalog_and_ic_paths(current_dir: PathBuf) -> Result<CatalogInfo, CatalogError> {
    println!("{:?}", current_dir);
    let mut cab_files = Vec::new();
    let mut exe_files = Vec::new();
    for entry in WalkDir::new(current_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let file_path = entry.path().to_path_buf();
        // let f_name = String::from(entry.path().to_string_lossy());
        if is_cab_path(&file_path) {
            cab_files.push(file_path);
        } else if is_ic_path(&file_path) {
            exe_files.push(file_path);
        }
    }
    println!("get_catalog_and_ic_paths--{:?}-{:?}", cab_files, exe_files);
    match (cab_files.as_slice(), exe_files.as_slice()) {
        ([cab_file], [exe_file]) => Ok((cab_file.clone(), exe_file.clone()).into()),
        ([cab_file], []) => Ok((cab_file.clone(), "No invc.exe file found".into()).into()),
        ([], [exe_file]) => Ok(("No .cab file found".into(), exe_file.clone()).into()),
        ([], []) => Err(CatalogError::CurrentFileError(
            "No .cab file and invc.exe file found".into(),
        )),
        (cab_files, exe_files) if cab_files.len() > 1 || exe_files.len() > 1 => Err(
            CatalogError::CurrentFileError("Multiple .cab and invc.exe files found".into()),
        ),
        (_, _) => Err(CatalogError::Unexpected),
    }
}

pub fn cab_to_xml(cab_path: &PathBuf) -> Result<PathBuf, CatalogError> {
    let cmd_str = format!("expand.exe -R {}", cab_path.to_string_lossy());
    println!("cab_to_xml-cmd_str: {}", cmd_str);
    let output = Command::new("cmd")
        // .creation_flags(CREATE_NO_WINDOW.0)
        .arg("/c")
        .arg(cmd_str)
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("cmd exec error: {}", e)))?;

    if !output.status.success() {
        return Err(CatalogError::ParseError("cmd command failed".into()));
    }

    let xml_path = format!(
        "{}.xml",
        filename_to_lower_string(cab_path).trim_end_matches(".cab")
    );
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
                        XmlEvent::EndElement { name: _ } => {
                            match event_writer.write(w_event) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Error: EndElement-else-event{}", e)
                                }
                            };
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
    value: &str,
) -> bool {
    let _value_name = str_to_pcwstr(value_name);
    let value = value
        .encode_utf16()
        .into_iter()
        .flat_map(|x| x.to_le_bytes())
        .collect::<Vec<u8>>();
    println!("set_reg_vaule--{:?}", value);
    let res = unsafe { Registry::RegSetValueExW(sub_key, _value_name, 0, dw_type, Some(&value)) };
    if res.is_ok() {
        unsafe {
            let _ = Registry::RegCloseKey(sub_key);
        };
        true
    } else {
        eprintln!("set_reg_vaule--error");
        false
    }
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
            r#"{{"CatalogHashValues":[{{"Key":{:?},"Value":"{}"}}]}}"#,
            xml_path, temp
        ),
        None => format!(
            r#"{{"CatalogHashValues":[{{"Key":{:?},"Value":"{}"}}]}}"#,
            xml_path, base64
        ),
    };
    println!("get_hash_sha384--{}", base64_str);
    Ok(base64_str)
}

pub fn handle_reg(str_hash: &str, software: &Software) {
    match software {
        Software::DellUpdate { app_name } => {
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
            open_software(app_name);
        }
        Software::DellCommandUpdate { app_name } => {
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
            open_software(app_name);
        }
    }
}

pub fn open_software(name: &String) {
    let mut enigo = Enigo::new();

    enigo.key_down(Key::Meta); // 按下 Windows 键
    enigo.key_click(Key::Layout('s')); // 按下 S 键
    enigo.key_up(Key::Meta); // 释放 Windows 键

    thread::sleep(Duration::from_millis(500));

    enigo.key_sequence(app_name);

    thread::sleep(Duration::from_millis(500));

    enigo.key_click(Key::Return);
}

pub enum Software {
    DellUpdate { app_name: String },
    DellCommandUpdate { app_name: String },
}

const DCU_PATH: &str = r#"SOFTWARE\Dell\UpdateService\Clients\CommandUpdate"#;
const DU_PATH: &str = r#"SOFTWARE\Dell\UpdateService\Clients\Update"#;
pub fn du_or_dcu() -> Option<Software> {
    match open_reg_subkey(DCU_PATH) {
        Ok(_) => Some(Software::DellCommandUpdate {
            app_name: "Dell Command Update".to_string(),
        }),
        Err(_) => match open_reg_subkey(DU_PATH) {
            Ok(_) => Some(Software::DellUpdate {
                app_name: "Dell Update".to_string(),
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

pub fn check_catalog_info(catalog_info: &CatalogInfo) -> Result<(), CatalogError> {
    print!("handle catalog_info--{:?}", catalog_info);
    if !is_cab_path(&catalog_info.cab_path) {
        return Err(CatalogError::SelectedFileError(format!(
            "Invalid file type",
        )));
    }
    if !is_ic_path(&catalog_info.ic_path) {
        return Err(CatalogError::SelectedFileError(format!(
            "Invalid file type",
        )));
    }
    Ok(())
}

pub async fn handle(catalog_info: &CatalogInfo) -> Result<(), CatalogError> {
    let xml_path = cab_to_xml(&PathBuf::from(&catalog_info.cab_path))?;
    let new_xml_path = handle_xml(xml_path)?;
    let hash = get_hash_sha384(new_xml_path)?;
    let _ = copy(
        &catalog_info.ic_path,
        r"C:\Program Files (x86)\Dell\UpdateService\Service\InvColPC.exe",
    );
    if let Some(ref software) = du_or_dcu() {
        match software {
            Software::DellUpdate { app_name } => {
                println!("{}", name);
                handle_reg(&hash, software)
            }
            Software::DellCommandUpdate { app_name } => {
                println!("{}", name);
                handle_reg(&hash, software)
            }
        }
    } else {
        println!("请安装DCU或者DU");
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cab_to_xml() {
        let xml_path = PathBuf::from("C:\\Users\\LJZ\\dev\\catalog-rs\\Precision_0CBB.cab");
        let result = cab_to_xml(&xml_path);
        assert!(result.is_ok());
    }
}
