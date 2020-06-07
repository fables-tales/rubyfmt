#!/usr/bin/env bash
set -e
if [ $# -eq 0 ]
  then
    echo "Please pass a file location to install the shim script"
    exit 1
fi

echo "#!/usr/bin/env bash
export RUBYFMT_USE_RELEASE=1
$(which ruby) --disable=all $(pwd)/rubyfmt.rb \$@" > $1
chmod +x $1