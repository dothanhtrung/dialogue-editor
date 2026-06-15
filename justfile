#!/usr/bin/env just --justfile

VERSION := `cargo pkgid | sed 's/.*#//'`

linux:
    cargo build --release
    rm -rf output/linux
    mkdir -p output/linux/dialogue-editor
    cp target/release/dialogue-editor output/linux/dialogue-editor/
    cd output/linux && tar cJvf ../dialogue-editor_{{VERSION}}.linux.x86-64.tar.xz dialogue-editor

windows:
    cargo build --target=x86_64-pc-windows-gnu --release
    rm -rf output/windows
    mkdir -p output/windows/dialogue-editor
    cp target/x86_64-pc-windows-gnu/release/dialogue-editor.exe output/windows/dialogue-editor/
    cd output/windows && zip -r ../dialogue-editor_{{VERSION}}.windows.x86-64.zip dialogue-editor

release: windows linux
