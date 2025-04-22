# ni-rs: Image Normalization CLI

This is the core image normalization tool used by the `ni-watcher` Windows service.
It processes images by cropping, resizing, and centering them onto a white canvas.

---

## 🔧 Build Instructions

### Prerequisites
- Rust (MSVC toolchain recommended)
- `image` crate for image processing

### Build
```cmd
cargo build --release
```

Output binary:
```
target\release\ni.exe
```

---

## 🚀 Usage
```cmd
ni.exe -i "input_path_or_folder" -o "output_folder" [-s "800 800"] [-t 10] [-p 50]
```

### Options
- `-i`, `--input`: Input image or folder
- `-o`, `--output`: Output directory (normalized image will be saved here)
- `-s`, `--size`: Optional. Target size as width and height (default: 800x800)
- `-t`, `--tolerance`: Optional. Tolerance for bounding box detection (default: 10)
- `-p`, `--padding`: Optional. Padding around centered object (default: 50)

### Example
```cmd
ni.exe -i "C:\images\raw" -o "C:\images\normalized" -s "1000 1000" -t 15 -p 40
```

---

## 💡 Features
- Auto-detects bounding box of non-white pixels
- Resizes and centers the content
- Supports individual files or entire folders
- Handles custom size, padding, and tolerance

---

## 🔗 Related Projects
- [`ni-service`](https://github.com/YOUR_USERNAME/ni-watcher): Windows service that uses `ni.exe` to normalize new images automatically

---

## 📃 License
MIT
