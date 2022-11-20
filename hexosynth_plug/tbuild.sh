cd ..
./copy_wlcode.sh
cd nih_plug
cargo +nightly xtask bundle hexosynth_plug --release
cp -vfr target/bundled/hexosynth_plug.vst3 ~/.vst3/
cp -vfr target/bundled/hexosynth_plug.clap ~/.vst3/
