import clr  # type: ignore  # noqa: F401
import json
import logging
import os
import pathlib
import shutil
import subprocess
import sys
import time
import winreg as registry
import xml.etree.ElementTree as ET
from pywinauto import Desktop, keyboard  # type: ignore
from typing import Final
from colorama import Fore, Style  # type: ignore
from System.Security.Cryptography import SHA384CryptoServiceProvider  # type: ignore
from System.IO import File, FileMode  # type: ignore
from System import Convert  # type: ignore


def run_cmd(command, delay_time=0):
    if delay_time > 0:
        time.sleep((5))
    return subprocess.run(command, shell=True, capture_output=True, text=True)


def open_dcu_du(app: str, catalog):
    print(Fore.GREEN + "change window regedit successful" + Style.RESET_ALL)
    ans = input(
        "Do you want do again? Please enter (a/b/other key) \n\t(a:change regedit again)\n\t(q:quit):\n\t(other key:open Dell Update) #"
    )
    if ans.lower() == "q":
        sys.exit(0)
    if ans.lower() == "a":
        handle_reg(app, catalog)
    keyboard.send_keys("{VK_LWIN down}" "s" "{VK_LWIN up}")
    if " " in app:
        _app = app.replace(" ", "{SPACE}")
        keyboard.send_keys(_app)
    keyboard.send_keys("{ENTER}")
    desktop = Desktop(backend="uia")
    if app == "Dell Command Update":
        up = desktop["Dell Command Update"]
    elif app == "Dell Update":
        up = desktop["Dell Update"]
    welcome_view = up.child_window(auto_id="update").child_window(
        class_name="ScrollViewer"
    )
    check_button = welcome_view.child_window(title="CHECK", control_type="Button")
    check_button.wait("ready", timeout=5)
    check_button.click()


def open_reg_key(
    sub_key,
):
    key = None
    try:
        key = registry.OpenKeyEx(
            registry.HKEY_LOCAL_MACHINE, sub_key, 0, registry.KEY_ALL_ACCESS
        )
    except Exception as e:
        logging.error("open_reg_key: " + str(e))
    return key


def set_reg_vaule(key, value_name: str, type: Final, vaule):  # type: ignore
    try:
        registry.SetValueEx(key, value_name, 0, type, vaule)
    except Exception as e:
        logging.error("set_reg_vaule: " + str(e))
    finally:
        registry.CloseKey(key)


def delete_reg_key_vaule(key, sub_key, value_names: list = []):
    try:
        if sub_key is not None:
            registry.DeleteKey(key, "IgnoreList")
        else:
            for vaule_name in value_names:
                registry.DeleteValue(key, vaule_name)
                run_cmd("taskkill /f /im %s" % rf"ServiceShell.exe > {os.devnull}")
                shutil.rmtree(r"C:\ProgramData\Dell\UpdateService\Temp")
    except Exception as e:
        logging.info("delete_reg_key_vaule: " + str(e))


def handle_cab() -> str:  # 需要管理员运行
    length = 0
    current_dir = os.getcwd()
    catalog_xml_path = ""
    while length < 1:
        files = [f for f in os.listdir(current_dir) if f.endswith(".cab")]
        length = len(files)
        if length == 1:
            catalog_cab = os.path.join(current_dir, files[0])
            run_cmd(rf'expand.exe -R "{catalog_cab}" > {os.devnull}')
            sp_catalog = catalog_cab.split(".")
            sp_catalog[-1] = ".xml"
            catalog_xml_path = "".join(sp_catalog)
            print(Fore.GREEN + "catalog cab: {}".format(catalog_cab) + Style.RESET_ALL)
            tree = ET.parse(catalog_xml_path)
            root = tree.getroot()
            root.set("baseLocation", "")
            namespace = "{openmanage/cm/dm}"
            iter_root = tree.iter(namespace + "SoftwareComponent")
            for node in iter_root:
                path = node.get("path").split("/")[-1]
                node.set("path", path)
            ET.register_namespace("", "openmanage/cm/dm")
            tree.write(catalog_xml_path, encoding="utf-8", xml_declaration=True)
        elif length == 0:
            input(
                "Place the catalog under the same directory and press <Enter>Continue"
            )
        else:
            input("Don't put more than one catalog file and press <Enter>Continue")
    return catalog_xml_path


def handle_reg(app: str, catalog):
    SHA384Provider = SHA384CryptoServiceProvider()
    dct = {}
    result = {}
    result["CatalogHashValues"] = []
    dct["Key"] = str(catalog)
    f = File.Open(catalog, FileMode.Open)
    hash = SHA384Provider.ComputeHash(f)
    val = Convert.ToBase64String(hash).strip("=")
    dct["Value"] = val
    result["CatalogHashValues"].append(dct)
    f.Close()
    result = json.dumps(result)
    Service_key = open_reg_key(r"SOFTWARE\Dell\UpdateService\Service")
    set_reg_vaule(Service_key, "CustomCatalogHashValues", registry.REG_SZ, result)
    service_vaule = [
        "LastCheckTimestamp",
        "LastUpdateTimestamp",
        "CatalogTimestamp",
        "CatalogTimestamp",
    ]
    delete_reg_key_vaule(Service_key, "IgnoreList")
    delete_reg_key_vaule(Service_key, None, service_vaule)
    for ic in pathlib.Path(f"{os.getcwd()}").rglob("inv*.exe"):
        if ic is not None:
            logging.info("handle_reg: " + str(ic))
            shutil.copy(
                str(ic),
                r"C:\Program Files (x86)\Dell\UpdateService\Service\InvColPC.exe",
            )
        else:
            print(
                Fore.RED + "Please put InvColPC.exe in current folder" + Style.RESET_ALL
            )
    if app == "Dell Command Update":
        cilent_key = open_reg_key(
            r"SOFTWARE\Dell\UpdateService\Clients\CommandUpdate\Preferences\Settings\General"
        )
        set_reg_vaule(
            cilent_key, "CustomCatalogPaths", registry.REG_MULTI_SZ, [catalog]
        )
        set_reg_vaule(cilent_key, "EnableCatalogXML", registry.REG_DWORD, 0x00000001)
        open_dcu_du("Dell Command Update", catalog)
    elif app == "Dell Update":
        cilent_key = open_reg_key(
            r"SOFTWARE\Dell\UpdateService\Clients\Update\Preferences\Settings\General"
        )
        set_reg_vaule(
            cilent_key, "CustomCatalogPaths", registry.REG_MULTI_SZ, [catalog]
        )
        set_reg_vaule(cilent_key, "EnableCatalogXML", registry.REG_DWORD, 0x00000001)
        open_dcu_du("Dell Update", catalog)
    else:
        print(Fore.RED + "请安装DU/DCU!" + Style.RESET_ALL)


def dcu_du() -> str:
    dcu_path = r"SOFTWARE\DELL\UpdateService\Clients\CommandUpdate"
    du_path = r"SOFTWARE\DELL\UpdateService\Clients\Update"
    if open_reg_key(dcu_path) is not None:
        return "Dell Command Update"
    elif open_reg_key(du_path) is not None:
        return "Dell Update"
    else:
        return ""


if __name__ == "__main__":
    logging.basicConfig(filename="catalog.log", level=logging.CRITICAL)
    logging.debug("Started")
    catalog = handle_cab()
    logging.debug("Finished")
    app = dcu_du()
    handle_reg(app=app, catalog=catalog)
