#!/bin/sh
bindgen wrapper.h -o ../src/bindings.rs -- -I../brotli/c/include
