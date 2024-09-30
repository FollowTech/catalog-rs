use data_encoding::BASE64;
use sha3::{Digest, Sha3_384};
use std::{
    fs::{copy, File, OpenOptions},
    io::{self, BufReader, BufWriter},
    process::Command,
};
use walkdir::WalkDir;
use windows::{
    core::PCWSTR,
    Win32::System::Registry::{
        self, HKEY, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, REG_SZ, REG_VALUE_TYPE,
    },
};
use xml::{attribute::Attribute, reader::EventReader, writer::XmlEvent, EventWriter};
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn get_catalog_path() -> Option<Vec<String>> {
    let mut cab_ic = Vec::new();
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let f_name = String::from(entry.file_name().to_string_lossy());
        if f_name.contains(".cab") {
            cab_ic.push(f_name.clone());
        } else {
            cab_ic.push("".to_string());
        }
        if f_name.to_lowercase().contains("invcolpc") && f_name.to_lowercase().contains(".exe") {
            cab_ic.push(f_name);
        } else {
            cab_ic.push("".to_string());
        }
        return Some(cab_ic);
    }
    return None;
}

pub fn cab_to_xml(cab_path: &str) -> std::io::Result<String> {
    let cmd_str = format!("expand.exe -R {cab_path} > nul ");
    Command::new("cmd")
        .arg("/c")
        .arg(cmd_str)
        .output()
        .expect("cmd exec error!");

    let xml_path = cab_path
        .split(".")
        .map(|cab: &str| if cab.contains("cab") { &".xml" } else { cab })
        .collect::<Vec<&str>>()
        .join("");
    // 读取原始 XML 文件
    let input_file = match File::open(&xml_path) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    let input_reader = BufReader::new(input_file);

    // 打开输出文件以写入
    let output_file = match OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&xml_path)
    {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let reader = EventReader::new(input_reader);

    let mut event_writer = EventWriter::new(BufWriter::new(output_file));

    let mut in_software = false;

    for event in reader.into_iter() {
        match event {
            Ok(event) => {
                if let Some(w_event) = event.as_writer_event() {
                    match &w_event {
                        XmlEvent::StartElement {
                            name,
                            attributes,
                            namespace,
                        } => {
                            if name.local_name == "software" {
                                in_software = true;

                                // 修改 path 属性
                                let mut new_attributes = Vec::new();
                                for attr in attributes.clone().into_owned() {
                                    if attr.name.local_name == "path" {
                                        let new_value =
                                            attr.value.split("\\").nth(1).unwrap_or_default();
                                        new_attributes.push(Attribute::new(attr.name, new_value));
                                    } else {
                                        new_attributes.push(attr);
                                    }
                                }

                                match event_writer.write(XmlEvent::StartElement {
                                    name: *name,
                                    attributes: new_attributes.into(),
                                    namespace: std::borrow::Cow::Borrowed(namespace),
                                }) {
                                    Ok(_) => {}
                                    Err(e) => eprintln!("Error: {}", e),
                                };
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
                                    match event_writer.write(w_event) {
                                        Ok(_) => {}
                                        Err(e) => eprintln!("Error: {}", e),
                                    };
                                } else {
                                    match event_writer.write(w_event) {
                                        Ok(_) => {}
                                        Err(e) => eprintln!("Error: {}", e),
                                    };
                                }
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

trait StrToPCWSTR {
    fn str_to_pcwstr(&self) -> PCWSTR;
}

impl StrToPCWSTR for &str {
    fn str_to_pcwstr(&self) -> PCWSTR {
        let result = self
            .to_string()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        PCWSTR::from_raw(result.as_ptr())
    }
}

pub fn open_reg_subkey(sub_key: &str) -> Result<HKEY, String> {
    let mut new_key: HKEY = HKEY::default();
    let _sub_key = sub_key.str_to_pcwstr();
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
    let _value_name = value_name.str_to_pcwstr();

    unsafe { Registry::RegSetValueExW(sub_key, _value_name, 0, dw_type, vaule.to_option_u8()) }
        .is_ok()
}

pub fn delete_reg_key_vaule(key: HKEY, sub_key: Option<&str>, value_names: Vec<&str>) {
    match sub_key {
        Some(_sub_key) => unsafe {
            let _ = Registry::RegDeleteKeyExW(key, _sub_key.str_to_pcwstr(), 0x0100, 0);
        },
        None => {
            for lpvaluename in value_names {
                unsafe {
                    let _ = Registry::RegDeleteValueW(key, lpvaluename.str_to_pcwstr());
                };
            }
        }
    }
}

pub fn get_hash_sha384(xml_path: &str) -> Result<Option<String>, std::io::Error> {
    let file = File::open(xml_path)?;
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

pub fn handle_reg(str_hash: &str) {
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
    if let Some(cab_and_ic) = get_catalog_path() {
        if cab_and_ic[1] == "".to_string() {
            let _ = copy(
                cab_and_ic[1].clone(),
                r"C:\Program Files (x86)\Dell\UpdateService\Service\InvColPC.exe",
            );
        } else {
            println!("Please put InvColPC.exe in current folder");
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let cab_path = get_catalog_path().unwrap()[0].clone();
        let xml = cab_to_xml(&cab_path);
    }
}
