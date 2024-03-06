<div align="center">
<img src="https://gitee.com/wanghoi/kwui/raw/master/icon.svg" height="140px" />

# kwui

使用 JSX、CSS 构建简单的桌面应用。 

</div>


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

## 快速开始

- 运行示例
  ```bash
  git clone https://gitee.com/wanghoi/kwui-rs.git
  cargo run --example installer
  ```
- 集成到您的Rust项目
  ```bash
  cargo add kwui
  ```

## 特点

- 简单实用
  - 适合快速开发，支持热重载
  - 易于使用Rust语言开发业务逻辑
  - 小巧无第三方依赖
- 兼容性好
  - 基于Direct2D实现GPU加速，电脑配置过低、GPU不可用时回落到软件渲染
  - 在生产环境，成千上万台电脑上使用
  - 优秀的多显示器与DPI缩放支持
- 脚本驱动：
  - 基于QuickJS扩展，原生JSX支持，i18n支持
  - 类似React Hooks的组件生命周期处理
- 布局功能丰富，样式美观
  - 符合 CSS 2.2 标准的排版与样式
  - 强大的图文混排，富文本支持
  - 中文输入法支持
- 易于扩展
  - 为原生渲染场景，如视频渲染做了优化
  - 支持JavaScript，C++，Rust编写业务逻辑
  - 支持C++扩展DOM，处理原生事件和渲染

## 样例展示

### 通话质量测试工具
![image](https://gitee.com/wanghoi/kwui/raw/master/docs/VoIPTool.png)

### 远程桌面客户端
![image](https://gitee.com/wanghoi/kwui/raw/master/docs/KuDesk.jpg)

### 安装程序
![image](https://gitee.com/wanghoi/kwui/raw/master/docs/installer.png)

## 常见问题

1. 已经有很多界面库了，为什么还要重复造轮子？
   
    为了解决工作中的问题：
    - 开发安装程序，需要界面美观，为了国际化需要自适应布局。
    - 多进程重构，需要UI代码编写一次，处处复用。
    - 音视频开发，需要便于C++扩展。
    - 业务逻辑使用Rust语言开发，需要为其提供界面。

    因为有趣：
    - 学习浏览器标准。
    - 学习Flutter, Electron, AWTK。
