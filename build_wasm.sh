# Variables:
publish_folder="./Reaction-resonance-release"
wasm_binary_name="reaction-resonance-eframe"
export OPENSSL_DIR="/usr/bin/openssl"

cargo build --release --target wasm32-unknown-unknown   #cargo builds the WASM binary using the release profile options
wasm-bindgen --no-typescript --target web --out-dir $publish_folder --out-name $wasm_binary_name ./target/wasm32-unknown-unknown/release/$wasm_binary_name.wasm # wasm-bindgen generates the javascript glue code for the WASM binary so it can be used in a static page
wasm-opt -O -ol 100 -s 100 -o $publish_folder'/'$wasm_binary_name'_bg.wasm' $publish_folder'/'$wasm_binary_name'_bg.wasm'	# optimizing the WASM binary for speed and size