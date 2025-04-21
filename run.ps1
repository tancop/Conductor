if (!(Test-Path "C:\Program Files (x86)\Steam\.cef-enable-remote-debugging")) {
    # enable Steam remote debugging
    Start-Process -Verb RunAs powershell.exe -Args "-NoProfile -ExecutionPolicy Bypass -Command `"New-Item -Path 'C:\Program Files (x86)\Steam\.cef-enable-remote-debugging' -ItemType File`""
}

python .\main.py