use data_encoding::BASE64;
use sha3::{Digest, Sha3_384};
use std::{
    collections::HashMap, ffi::c_char, fs::File, io::{self, BufReader}, process::Command, ptr::null_mut
};
use walkdir::WalkDir;
use windows::{
    core::PCWSTR,
    Win32::System::Registry::{self, HKEY, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, REG_SZ, REG_VALUE_TYPE},
};
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

pub fn cab_to_xml(cab_path: &String) -> String {
    let cmd_str = format!("expand.exe -R {cab_path} > nul ");
    Command::new("cmd")
        .arg("/c")
        .arg(cmd_str)
        .output()
        .expect("cmd exec error!");
    cab_path
        .split(".")
        .map(|cab: &str| if cab.contains("cab") { &".xml" } else { cab })
        .collect::<Vec<_>>()
        .join("")
    // let sp_cab = cab_path.splitn(2, ".").collect::<Vec<_>>();
    // format!("{}{}", sp_cab[0], ".xml")
}

pub fn open_reg_key(sub_key: PCWSTR) -> Result<*mut HKEY, ()> {
    let key = null_mut::<HKEY>();
    let l_result =
        unsafe { Registry::RegOpenKeyExW(HKEY_LOCAL_MACHINE, sub_key, 0, KEY_ALL_ACCESS, key) };
    if l_result.is_ok() {
        return Ok(key);
    } else {
        Err(())
    }
}

pub fn set_reg_vaule(
    sub_key: PCWSTR,
    value_name: PCWSTR,
    dw_type: REG_VALUE_TYPE,
    vaule: Option<&[u8]>,
) -> bool {
    unsafe {
        Registry::RegSetValueExW(
            HKEY_LOCAL_MACHINE,
            sub_key,
            0,
            dw_type,
            lp_data,
            vaule,
        )
        .is_ok()
    }
}

pub fn get_hash_sha384(xml_path: &String) -> io::Result<()> {
    let file = File::open(xml_path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha3_384::new();
    io::copy(&mut reader, &mut hasher)?;
    let hash = hasher.finalize();
    if let Some(base64_str) = BASE64.encode(&hash[..]).strip_suffix("=") {
        let json = format!(r#"{{"Key":{},"Value":{}\}}"#, xml_path, base64_str);
        let p_json = Some(json as )
        set_reg_vaule(
            PCWSTR::from_raw(r"SOFTWARE\Dell\UpdateService\Service".as),
            PCWSTR::from_raw("CustomCatalogHashValues".as_ptr()),
            REG_SZ.0,
            p_json,
            json.len()
        );
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
