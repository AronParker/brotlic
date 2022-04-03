#!/bin/sh
bindgen wrapper.h -o src/bindings.rs -- -Ibrotli/c/include
