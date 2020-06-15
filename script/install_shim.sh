#!/usr/bin/env bash
set -e
if [ $# -eq 0 ]
  then
    echo "Please pass a file location to install the shim script"
    exit 1
fi

rubyfmt_root="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../" >/dev/null 2>&1 && pwd )"

echo "#!/usr/bin/env bash
export RUBYFMT_USE_RELEASE=1
$(which ruby) --disable=all $rubyfmt_root/rubyfmt.rb \$@" > "$1"
chmod +x "$1"
