#!/bin/bash
set -e

echo "Generating bad_app.ipa..."
cd "$(dirname "$0")"

# Build BadApp IPA
cp -r Payload/BadApp.app /tmp/BadApp.app
mkdir -p /tmp/bad/Payload
mv /tmp/BadApp.app /tmp/bad/Payload/
cd /tmp/bad
zip -r bad_app.ipa Payload > /dev/null
mv bad_app.ipa "$OLDPWD"/bad_app.ipa
rm -rf /tmp/bad

echo "Generating good_app.ipa..."
cd "$OLDPWD"
# Build GoodApp IPA
cp -r Payload/GoodApp.app /tmp/GoodApp.app
mkdir -p /tmp/good/Payload
mv /tmp/GoodApp.app /tmp/good/Payload/
cd /tmp/good
zip -r good_app.ipa Payload > /dev/null
mv good_app.ipa "$OLDPWD"/good_app.ipa
rm -rf /tmp/good

echo "Done!"
ls -la "$OLDPWD/"*.ipa
