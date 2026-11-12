```bash
rustup target add aarch64-linux-android

cargo install cargo-ndk
```

## Windows

```
$env:ANDROID_NDK_HOME = "your_ANDROID_NDK_HOME_path"

cargo ndk -t arm64-v8a build --release
```

## Deployment

```bash
adb push target\aarch64-linux-android\release\xyd /data/local/tmp/
```

```bash
adb shell
su
chmod +x /data/local/tmp/xyd

./data/local/tmp/xyd
```
