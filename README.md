<div align="center">
<img src="https://gitee.com/wanghoi/kwui/raw/master/icon.svg" height="140px" />

# kwui-rs

A small user interface library for daily use.
Build utility Gui tool with `JSX`、`CSS` and `Rust`.

</div>

## Hello world

```javascript
import { useState } from "Keact";

function HelloWorld(props, kids) {
    let [n, setN] = useState(0);
    return <button onclick={() => setN(n + 1)}>{`Click ${n} times`}</button>;
}

app.showDialog({
    title: "Hello World",
	root: <HelloWorld />,
	stylesheet: css`
	button { margin: 10px; padding: 4px; background-color: orange; }
	button:hover { background-color: orangered; }
    `
});
```

## Quick Start

1. Fetch source code
```bash
git clone https://gitee.com/wanghoi/kwui-rs.git
```
2a. Run Win32 example
```bash
# Run the mock installer 
cargo run -p installer
```
2b. Or build Android example
```bash
# Build the mock installer 
cmake --preset android-debug
cmake --build --preset android-debug --target installer.APK
```

## Gallery

### VoIP QoE tool
![image](docs/VoIPTool.png)

### Remote Desktop
![image](docs/KuDesk.jpg)

### Installer
![image](docs/installer.png)
![image](docs/installer-android.jpg)

### Richtext
![image](docs/richtext-android.jpg)


## Documentation
- [TODO: API Reference](https://github.com/wanghoi/kwui-rs/wikis)

## FAQ

1. Why another GUI library?
- Porting server-side Rust code to client-side quickly, to explore and evaluate new technology.
- Explore end-to-end and server-relay audio and video transport technology.
- Explore SDWAN technology.
