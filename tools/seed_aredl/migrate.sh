#!/bin/sh
set -e

if [ -z "$(ls -A ${AREDL_DATA_PARENT} 2>/dev/null)" ]; then
  echo "Cloning repository..."
  git clone --depth 1 -b main "${AREDL_DATA_GIT}" ${AREDL_DATA_PARENT}
else
  echo "Repository already exists; updating..."
  cd ${AREDL_DATA_PARENT} && git pull origin main
fi

export AREDL_DATA_PATH="${AREDL_DATA_PARENT}/data"

echo "Running migration..."
exec seed_aredl