#!/bin/bash

if [ $# -lt 2 ]; then
	echo "Usage: $0 <register file name> <unregister file name>"
	exit 1
fi

if [ -z "${APPNAME}" ]; then
	exit 1
fi

REGFILE="$1"
UNREGFILE="$2"
UUID="$(uuidgen)"

cat << EOF > ${REGFILE}
Windows Registry Editor Version 5.00

[HKEY_CLASSES_ROOT\\Applications\\${APPNAME}.exe]
"AppUserModelID"="${AUMID}"

[HKEY_CURRENT_USER\\Software\\Classes\\AppUserModelId\\${AUMID}]
"DisplayName"="${APPNAME}"
"ToastActivatorCLSID"="{${UUID}}"

[HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Notifications\\Settings\\${APPNAME}]
"Enabled"=dword:00000001
EOF

echo "Generated ${REGFILE} with AUMID=${AUMID} and UUID=${UUID}"

cat << EOF > ${UNREGFILE}
Windows Registry Editor Version 5.00

[-HKEY_CLASSES_ROOT\\Applications\\${APPNAME}.exe]

[-HKEY_CURRENT_USER\\Software\\Classes\\AppUserModelId\\${AUMID}]

[-HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Notifications\\Settings\\${APPNAME}]
EOF

echo "Generated ${UNREGFILE}"
