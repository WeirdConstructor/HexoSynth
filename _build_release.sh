mkdir -p release
rm release/hexosynth_jack
rm release/hexosynth_vst.so

cd jack_standalone
cargo build --release
cp target/release/hexosynth_jack ../release/
cd ..

cd vst2
cargo build --release
cp target/release/libhexosynth_vst.so ../release/hexosynth_vst.so
cd ..
