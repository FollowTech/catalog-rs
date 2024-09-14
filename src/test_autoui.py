# coding=utf-8
# 安装requirements,csg文件
import pathlib
import os
import subprocess

cwd = os.getcwd().replace("\\", "/")
pip_ini = r"""$pip = @"
[global]
index-url =  https://mirrors.aliyun.com/pypi/simple/
[install]
trusted-host=mirrors.aliyun.com/pypi/simple/
"@
New-item "$env:USERPROFILE\pip\pip.ini" -Value $pip -Force"""

stdout_line = subprocess.getoutput(f"pip list")
if "pywin32" not in stdout_line:
    subprocess.run(["powershell.exe", pip_ini])
    subprocess.run("python -m pip install pywin32==306 flask")


try:
    import ctypes
    import sys
    import os
    import shutil
    import time
    import win32con
    import win32clipboard
    import win32gui
    import win32api

except Exception as e:
    msg = f"Exception importing modules as : {str(e)}"
    print(msg)
    sys.exit(1)

cur_path = os.path.dirname(__file__)
cwd = os.getcwd().replace("\\", "/")


def run_cmd(command, delay_time=0):
    if delay_time > 0:
        time.sleep((5))
    return subprocess.run(command, shell=True, capture_output=True, text=True)


def add_user(uname, passwd, fname):
    result = run_cmd("net user %s" % uname)
    if result.returncode == 0:
        print("该用户已经存在")
        return

    run_cmd(
        [
            "powershell.exe",
            rf"New-LocalUser -Name {uname} -Password (ConvertTo-SecureString -AsPlainText {passwd} -Force)",
        ]
    )
    run_cmd(
        [
            "powershell.exe",
            rf"Set-LocalUser -Name {uname} -FullName '{fname}' -PasswordNeverExpires $true",
        ]
    )
    run_cmd("net localgroup administrators %s /add " % (uname))
    run_cmd("net localgroup users %s /del " % (uname))
    run_cmd("net accounts /maxpwage:unlimited")
    return


def copy_share_dir(src_path):
    dash_share = "C:\DashShare"
    if os.path.exists(dash_share):
        run_cmd("net share DashContent /delete")
        run_cmd("net share Logs /delete")
        shutil.rmtree(path=dash_share, ignore_errors=True)
    os.makedirs(dash_share)

    os.makedirs(f"{dash_share}\Logs")
    run_cmd(
        f"net share Logs={dash_share}\Logs /grant:DashService,Full /grant:DashAdmin,Read"
    )
    dash_content = f"{cur_path}\DashContent"
    if os.path.exists(dash_content):
        shutil.rmtree(path=dash_content, ignore_errors=True)
    os.makedirs(dash_content)

    if not os.path.isfile(src_path):
        cur_dir = os.path.split(src_path)[-1]
        new_path = os.path.join(os.path.abspath(f"{dash_share}"), cur_dir)
        shutil.copytree(src_path, new_path, dirs_exist_ok=True)
    else:
        shutil.copyfile(src_path, f"{dash_share}")
    run_cmd(
        f"net share DashContent={dash_share}\DashContent /grant:DashAdmin,Full /grant:DashService,Read"
    )
    return


def whether_to_install_docker():
    if (
        not run_cmd(
            [
                "powershell.exe",
                'Get-WindowsOptionalFeature -Online | Where-Object { $_.State -eq "Enabled" -and $_.FeatureName -eq "Microsoft-Hyper-V" }',
            ]
        ).stdout
        == ""
    ):
        run_cmd(
            [
                "powershell.exe",
                'Enable-WindowsOptionalFeature -Online -FeatureName $("Microsoft-Hyper-V", "Containers") -All -Norestart',
            ]
        )
    docker1 = os.path.exists(r"c:\program files\docker\docker\resources\dockerd.exe")
    docker2 = os.path.exists(r"c:\program files\docker\docker\resources\bin\docker.exe")
    return docker1 & docker2


def do_key_input(msg, clip_board_mode=True, key_sleep=0):
    if clip_board_mode:  # 剪贴板方式
        win32clipboard.OpenClipboard()
        win32clipboard.EmptyClipboard()
        win32clipboard.SetClipboardText(msg)
        win32clipboard.CloseClipboard()
        win32api.keybd_event(win32con.VK_CONTROL, 0, 0, 0)
        win32api.keybd_event(ord("V"), 0, 0, 0)
        win32api.keybd_event(win32con.VK_CONTROL, 0, win32con.KEYEVENTF_KEYUP, 0)
        win32api.keybd_event(ord("V"), 0, win32con.KEYEVENTF_KEYUP, 0)
    else:  # 按键输入方式
        for c in msg:
            time.sleep(key_sleep)
            if c == "!":
                win32api.keybd_event(win32con.VK_SHIFT, 0, 0, 0)
                win32api.keybd_event(49, 0, 0, 0)
                win32api.keybd_event(win32con.VK_SHIFT, 0, win32con.KEYEVENTF_KEYUP, 0)
                win32api.keybd_event(49, 0, win32con.KEYEVENTF_KEYUP, 0)
            elif c == ":":
                win32api.keybd_event(win32con.VK_SHIFT, 0, 0, 0)
                win32api.keybd_event(186, 0, 0, 0)
                win32api.keybd_event(win32con.VK_SHIFT, 0, win32con.KEYEVENTF_KEYUP, 0)
                win32api.keybd_event(186, 0, win32con.KEYEVENTF_KEYUP, 0)
            elif c == ",":
                win32api.keybd_event(188, 0, 0, 0)
                win32api.keybd_event(188, 0, win32con.KEYEVENTF_KEYUP, 0)
            elif c == ".":
                win32api.keybd_event(190, 0, 0, 0)
                win32api.keybd_event(190, 0, win32con.KEYEVENTF_KEYUP, 0)
            elif c == "/":
                win32api.keybd_event(191, 0, 0, 0)
                win32api.keybd_event(191, 0, win32con.KEYEVENTF_KEYUP, 0)
            elif c == "\\":
                win32api.keybd_event(220, 0, 0, 0)
                win32api.keybd_event(220, 0, win32con.KEYEVENTF_KEYUP, 0)
            else:
                code = ord(c)
                if code >= 97 and code <= 122:
                    code -= 32
                    win32api.keybd_event(code, 0, 0, 0)
                    win32api.keybd_event(code, 0, win32con.KEYEVENTF_KEYUP, 0)
                elif code >= 65 and code <= 90:
                    win32api.keybd_event(win32con.VK_SHIFT, 0, 0, 0)
                    win32api.keybd_event(code, 0, 0, 0)
                    win32api.keybd_event(
                        win32con.VK_SHIFT, 0, win32con.KEYEVENTF_KEYUP, 0
                    )
                    win32api.keybd_event(code, 0, win32con.KEYEVENTF_KEYUP, 0)
                else:
                    win32api.keybd_event(code, 0, 0, 0)
                    win32api.keybd_event(code, 0, win32con.KEYEVENTF_KEYUP, 0)


def do_keyEnter_input():
    win32api.keybd_event(0x0D, 0, 0, 0)
    win32api.keybd_event(0x0D, 0, win32con.KEYEVENTF_KEYUP, 0)
    time.sleep(1)


def mouse_click(x, y):
    win32api.SetCursorPos([x, y])
    win32api.mouse_event(win32con.MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0)
    win32api.mouse_event(win32con.MOUSEEVENTF_LEFTUP, 0, 0, 0, 0)


def wait_for_window(targetTitle, max_time=5):
    t = time.time()
    while time.time() - t < max_time:
        hWndList = []
        win32gui.EnumWindows(lambda hWnd, param: param.append(hWnd), hWndList)
        for hwnd in hWndList:
            try:
                clsname = win32gui.GetClassName(hwnd)  # noqa: F841
                title = win32gui.GetWindowText(hwnd)
                # file.write(title)
                # print(title)
                if title.find(targetTitle) >= 0:
                    win32gui.ShowWindow(hwnd, win32con.SW_SHOWNORMAL)
                    win32gui.SetForegroundWindow(hwnd)
                    win32gui.SetActiveWindow(hwnd)
            except Exception as ex:
                print(ex)
        time.sleep(0.2)
    return None


def install_dash():
    std_out = run_cmd("netsh interface show interface").stdout
    lines = std_out.strip().split("\n")
    network_data = [line.split() for line in lines]
    lan = None
    for data in network_data:
        if "Enabled" in data and "Connected" in data and ("Ethernet" in data):
            lan = data
            print(lan)
    if lan == None:
        print("\033[91mPlug in the wire network!\033[0m")
        time.sleep(3)
        sys.exit(0)
    text = " ".join(lan)
    index = text.find("Ethernet")
    network_name = text[index:]
    print(network_name)
    run_cmd(f'netsh interface ipv4 set address "{network_name}" source=dhcp')

    time.sleep(2)
    p = pathlib.Path(
        r"\\172.16.2.2\Tools\[ DASH Server ] SW & HW\DASH files\DASH 2.X Server Installation Package"
    )
    dashzip = ""
    for i in p.rglob("dash_server*install*.7z"):
        dashzip = i
    print("正在拉取Dashserver文件")
    run_cmd(r"cmdkey /delete:172.16.2.2 >nul")
    run_cmd(r"cmdkey /add:172.16.2.2 /user:User1 /pass:Us111111 >nul")
    run_cmd(rf'robocopy "{dashzip.parent}" {cur_path}')
    time.sleep(2)
    run_cmd(
        f'netsh interface ipv4 set address "{network_name}" static 192.168.1.2 255.255.255.0 192.168.1.2'
    )

    subprocess.Popen("cmd.exe /c" + f"{cur_path}\install.cmd")
    wait_for_window("C:\Windows\system32\cmd.exe", max_time=3)  # 最多等3秒钟

    do_key_input(r"\\192.168.1.2\DashContent", clip_board_mode=True, key_sleep=0.2)
    do_keyEnter_input()
    do_key_input("DashService", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    do_key_input("Dash123", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    time.sleep(2)
    do_key_input(r"\\192.168.1.2\Logs", clip_board_mode=True, key_sleep=0.2)
    do_keyEnter_input()
    do_key_input("DashService", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    do_key_input("Dash123", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    time.sleep(2)
    do_key_input("192.168.1.2", clip_board_mode=True, key_sleep=0.2)
    do_keyEnter_input()
    time.sleep(2)
    do_key_input("Dash Pro", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    time.sleep(2)
    do_key_input("WCD", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    time.sleep(2)
    do_key_input("10", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    time.sleep(2)
    do_key_input("Dash123", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    time.sleep(2)
    do_key_input("N", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    time.sleep(2)
    do_key_input("Y", clip_board_mode=False, key_sleep=0.2)
    do_keyEnter_input()
    # finsh


def is_admin():
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:  # noqa: E722
        pass
    finally:
        return False


print("admin_exe前")
if is_admin():
    print(cur_path)
    # 将要运行的代码加到这里
    print("以管理员权限运行")
    print(cur_path)
    if not whether_to_install_docker():
        print("\033[91m请安装Docker并切换到Windows容器!\033[0m")
        time.sleep(3)
        sys.exit(0)
    add_user("DashAdmin", "Dash123", "Dash Adminstrator")
    add_user("DashService", "Dash123", "Dash Service")
    time.sleep(5)
    copy_share_dir(f"{cur_path}\DashContent")

    install_dash()
else:
    if sys.version_info[0] == 3:
        print("没有以管理员权限运行")
        ctypes.windll.shell32.ShellExecuteW(
            None, "runas", sys.executable, __file__, None, 1
        )
    else:  # in python2.x
        ctypes.windll.shell32.ShellExecuteW(
            None,
            "runas",
            unicode(sys.executable),
            unicode(__file__),
            None,
            1,  # noqa: F821
        )
print("admin_exe后")
