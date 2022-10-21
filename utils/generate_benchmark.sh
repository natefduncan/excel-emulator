#!/bin/usr/env bash
hyperfine "./target/release/excel ./assets/test3.xlsx calculate Summary\!A1" > ./utils/benchmark.txt
