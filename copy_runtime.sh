./target/release/tiedye build-spec --dev >> tmp_dev.json;
node cpRuntime.js;
rm tmp_dev.json;
echo "Done"