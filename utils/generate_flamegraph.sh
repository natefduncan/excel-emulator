cargo build --release 
rm -rf ./utils/flame.svg && cd ./utils && perf record --call-graph dwarf,16384 -e cpu-clock -F 997 ../target/release/excel ../assets/test3.xlsx calculate Summary\!A1 
perf script | stackcollapse-perf.pl | stackcollapse-recursive.pl | c++filt | rust-unmangle | flamegraph.pl > ./flame.svg
rm -rf ./perf.data
